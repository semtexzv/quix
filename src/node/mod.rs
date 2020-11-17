use crate::import::*;

mod link;

use crate::node::link::NodeLink;
use crate::util::{RegisterRecipient, RpcMethod};
use crate::global::{Get, Global};
use crate::process::{Dispatcher, DispatchError};
use tokio::net::TcpStream;
use crate::{Broadcast, NodeDispatch, MethodCall};
use crate::process::registry::ProcessRegistry;

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: Uuid,
    pub listen: SocketAddr,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            id: Uuid::new_v4(),
            listen: ([127, 0, 0, 1], 9090).into(),
        }
    }
}

pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn send<M, T>(&self, m: M) -> impl Future<Output=M::Result>
    where M: RpcMethod + Message<Result=Result<T, DispatchError>>
    {
        let nodeid = self.0;
        async move {
            let res = NodeController::from_registry().send(NodeDispatch {
                nodeid,
                inner: m.make_call(None),
            }).await.map_err(|e| DispatchError::MailboxRemote)??;

            M::read_result(res)
        }
    }
    pub fn do_send<M>(&self, m: M)
    where M: RpcMethod
    {
        NodeController::from_registry().do_send(NodeDispatch {
            nodeid: self.0,
            inner: m.make_broadcast(None),
        })
    }
}

pub struct NodeController {
    /// Links to other nodes
    links: HashMap<Uuid, Addr<NodeLink>>,
    /// Dispatcher for unaddressed messages.
    ///
    /// Messages which are sent to process id of 00000000000000000....
    /// are considered unadressed, and are dispatched from here
    pub dispatch: HashMap<u32, Box<dyn CallHandler>>,
    pub status_listeners: HashMap<Uuid, Recipient<NodeStatus>>,
}

impl SystemService for NodeController {}

impl Supervised for NodeController {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        // Ensure process registry is initialized
        ProcessRegistry::from_registry();

        let cfg = Global::<NodeConfig>::from_registry().send(Get::default());
        let set_cfg = wrap_future(cfg)
            .then(|cfg, this: &mut Self, ctx| {
                let cfg = cfg.unwrap();
                log::warn!("Starting node listener on: {:?}", cfg);
                let cfg = cfg.unwrap();

                wrap_future(tokio::net::TcpListener::bind(cfg.listen))
            })
            .map(|listener, this: &mut Self, ctx| {
                let mut listener = listener.unwrap();
                ctx.add_stream(listener);
            });

        ctx.wait(set_cfg);
    }
}

impl Default for NodeController {
    fn default() -> Self {
        NodeController {
            links: HashMap::new(),
            dispatch: HashMap::new(),
            status_listeners: HashMap::new(),
        }
    }
}

impl Actor for NodeController {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        Self::restarting(self, ctx);
    }
}

impl StreamHandler<std::io::Result<TcpStream>> for NodeController {
    fn handle(&mut self, item: std::io::Result<TcpStream>, ctx: &mut Context<Self>) {
        let item = item.expect("Fatal error in NodeControl");

        let link = wrap_future(NodeLink::new(item));
        let fut = link
            .map(|(id, peer, link), this: &mut Self, ctx| {
                log::info!("Connected to: {:?}", id);
                this.links.insert(id, link.clone());
                this.status_listeners.retain(|_, l| {
                    l.do_send(NodeStatus::Connected(id)).is_ok()
                });
            });
        ctx.spawn(fut);
    }
}

#[derive(Debug, Clone)]
pub enum NodeStatus {
    Connected(Uuid),
    Disconnected(Uuid),
}

impl Message for NodeStatus { type Result = (); }

impl Handler<NodeStatus> for NodeController {
    type Result = ();

    fn handle(&mut self, msg: NodeStatus, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NodeStatus::Disconnected(id) => {
                self.status_listeners.retain(|_, l| l.do_send(msg.clone()).is_ok());
            }
            _ => panic!("Should not receive connected event yet")
        }
    }
}

impl Handler<RegisterRecipient<NodeStatus>> for NodeController {
    type Result = Result<Uuid, std::convert::Infallible>;

    fn handle(&mut self, msg: RegisterRecipient<NodeStatus>, ctx: &mut Self::Context) -> Self::Result {
        let id = Uuid::new_v4();
        self.status_listeners.insert(id, msg.0);
        return Ok(id);
    }
}


pub struct Connect {
    pub addr: SocketAddr
}

impl Message for Connect {
    type Result = Addr<NodeLink>;
}

impl Handler<Connect> for NodeController {
    type Result = ResponseActFuture<Self, Addr<NodeLink>>;

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        log::warn!("Connecting to remote node");

        // TODO: Retrying
        let conn = async move {
            let stream = TcpStream::connect(msg.addr).await.unwrap();
            NodeLink::new(stream).await
        };

        let link = wrap_future(conn);
        Box::pin(link.map(|(id, peer, link), this: &mut Self, ctx| {
            log::info!("Connected to: {:?}", id);
            this.links.insert(id, link.clone());
            link
        }))
    }
}

pub struct ListNodes;

impl Message for ListNodes {
    type Result = Vec<Uuid>;
}

impl Handler<ListNodes> for NodeController {
    type Result = ResponseFuture<Vec<Uuid>>;

    fn handle(&mut self, msg: ListNodes, ctx: &mut Context<Self>) -> Self::Result {
        return Box::pin(futures::future::ready(self.links.keys().cloned().collect()));
    }
}

// Per node broadcast, understood as calling default handler of method from node
impl Handler<NodeDispatch<MethodCall>> for NodeController {
    type Result = actix::Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: NodeDispatch<MethodCall>, ctx: &mut Self::Context) -> Self::Result {
        if let Some(link) = self.links.get(&msg.nodeid) {
            let link = link.clone();

            let send = link.send(msg.inner)
                .map(|r| r
                    .map_err(|_| DispatchError::MailboxLocal)
                    .and_then(|r| r)
                );

            actix::Response::fut(Box::pin(send))
        } else {
            return actix::Response::reply(Err(DispatchError::NodeNotFound));
        }
    }
}


impl Handler<NodeDispatch<Broadcast>> for NodeController {
    type Result = Result<(), DispatchError>;

    fn handle(&mut self, msg: NodeDispatch<Broadcast>, ctx: &mut Self::Context) -> Self::Result {
        if let Some(link) = self.links.get(&msg.nodeid) {
            let link = link.clone();
            link.do_send(msg.inner);
            Ok(())
        } else {
            Err(DispatchError::NodeNotFound)
        }
    }
}

// Global broadcast
impl Handler<Broadcast> for NodeController {
    type Result = Result<(), DispatchError>;

    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) -> Self::Result {
        assert!(msg.procid.is_none());

        for n in self.links.values() {
            n.do_send(msg.clone());
        }
        Ok(())
    }
}

// Remote node calling us
impl Handler<FromNode<MethodCall>> for NodeController {
    type Result = Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: FromNode<MethodCall>, ctx: &mut Self::Context) -> Self::Result {
        log::info!("NodeControl dispatching unadressed message");
        match self.dispatch.get_mut(&msg.inner.method) {
            Some(m) => {
                let fut = (m).handle(msg.node_id, msg.inner.body);
                Response::fut(fut)
            }
            None => {
                log::warn!("Message: {} not handled by any global handler", msg.inner.method);
                return Response::reply(Err(DispatchError::MethodNotFound));
            }
        }
    }
}

#[derive(Debug)]
pub struct FromNode<M> {
    pub node_id: Uuid,
    pub inner: M,
}

impl<M: Message> Message for FromNode<M> {
    type Result = M::Result;
}

// TODO: Remove this, unify dispatchers and handlers
pub trait CallHandler: Send {
    fn handle(&mut self, node_id: Uuid, data: Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>>;
}


impl<M, T> CallHandler for Recipient<FromNode<M>>
where M: Message<Result=Result<T, DispatchError>> + RpcMethod + Send + 'static,
      M::Result: Send
{
    fn handle(&mut self, node_id: Uuid, data: Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>> {
        let msg = M::read(data)
            .map_err(|_| DispatchError::MessageFormat)
            .map(|msg| FromNode {
                node_id,
                inner: msg,
            })
            .map(|msg| self.send(msg));

        Box::pin(async move {
            log::trace!("Running global message handler");
            let response = msg?.await.map_err(|_| DispatchError::MessageFormat)?;
            let mut buf = BytesMut::new();
            M::write_result(&response, &mut buf).map_err(|_| DispatchError::MessageFormat)?;
            Ok(buf.freeze())
        })
    }
}


impl<M, T> CallHandler for Recipient<M>
where M: Message<Result=Result<T, DispatchError>> + RpcMethod + Send + 'static,
      M::Result: Send
{
    fn handle(&mut self, node_id: Uuid, data: Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>> {
        let msg = M::read(data)
            .map_err(|_| DispatchError::MessageFormat)
            .map(|msg| self.send(msg));

        Box::pin(async move {
            log::trace!("Running global message handler");
            let response = msg?.await.map_err(|_| DispatchError::MessageFormat)?;
            let mut buf = BytesMut::new();
            M::write_result(&response, &mut buf).map_err(|_| DispatchError::MessageFormat)?;
            Ok(buf.freeze())
        })
    }
}

pub struct RegisterGlobalHandler {
    method: u32,
    handler: Box<dyn CallHandler>,
}

impl RegisterGlobalHandler {
    pub fn with_nodeinfo<M, T>(rec: Recipient<FromNode<M>>) -> Self
    where M: actix::Message<Result=Result<T, DispatchError>> + RpcMethod + Send + 'static,
          T: prost::Message + Send + 'static
    {
        RegisterGlobalHandler {
            method: M::ID,
            handler: Box::new(rec),
        }
    }
    /// Create new registration request for node-wide handler
    pub fn new<M, T>(rec: Recipient<M>) -> Self
    where M: actix::Message<Result=Result<T, DispatchError>> + RpcMethod + Send + 'static,
          T: prost::Message + Send + 'static
    {
        RegisterGlobalHandler {
            method: M::ID,
            handler: Box::new(rec),
        }
    }
}

impl Message for RegisterGlobalHandler {
    type Result = ();
}

impl Handler<RegisterGlobalHandler> for NodeController {
    type Result = ();

    fn handle(&mut self, msg: RegisterGlobalHandler, ctx: &mut Context<Self>) -> Self::Result {
        self.dispatch.insert(msg.method, msg.handler);
    }
}
