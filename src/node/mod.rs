use crate::import::*;


mod link;

use crate::node::link::NodeLink;
use crate::util::{RegisterRecipient, RpcMethod};
use crate::global::{Get, Global};
use crate::process::{DispatchError, Dispatcher};
use tokio::net::TcpStream;
use crate::{Broadcast, Dispatch, NodeDispatch, MethodCall};

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

pub struct NodeControl {
    /// Links to other nodes
    links: HashMap<Uuid, Addr<NodeLink>>,
    /// Dispatcher for unaddressed messages.
    ///
    /// Messages which are sent to process id of 00000000000000000....
    /// are considered unadressed, and are dispatched from here
    pub dispatch: HashMap<u32, NodeHandler>,
    pub status_listeners: HashMap<Uuid, Recipient<NodeStatus>>,
}

impl SystemService for NodeControl {}

impl Supervised for NodeControl {
    fn restarting(&mut self, ctx: &mut Self::Context) {
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

impl Default for NodeControl {
    fn default() -> Self {
        NodeControl {
            links: HashMap::new(),
            dispatch: HashMap::new(),
            status_listeners: HashMap::new(),
        }
    }
}

impl Actor for NodeControl {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        Self::restarting(self, ctx);
    }
}

impl StreamHandler<std::io::Result<TcpStream>> for NodeControl {
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

impl Handler<NodeStatus> for NodeControl {
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

impl Handler<RegisterRecipient<NodeStatus>> for NodeControl {
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

impl Handler<Connect> for NodeControl {
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

impl Handler<NodeDispatch<MethodCall>> for NodeControl {
    type Result = actix::Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: NodeDispatch<MethodCall>, ctx: &mut Self::Context) -> Self::Result {
        if let Some(link) = self.links.get(&msg.nodeid) {
            let link = link.clone();

            let work = async move {
                link.send(msg.inner).await.unwrap()
            };

            actix::Response::fut(Box::pin(work))
        } else {
            return actix::Response::reply(Err(DispatchError::DispatchRemote));
        }
    }
}


impl Handler<Broadcast> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) -> Self::Result {
        for n in self.links.values() {
            n.do_send(msg.clone());
        }
    }
}

impl Handler<FromNode<MethodCall>> for NodeControl {
    type Result = Result<(), DispatchError>;

    fn handle(&mut self, msg: FromNode<MethodCall>, ctx: &mut Self::Context) -> Self::Result {
        log::info!("NodeControl dispatching unadressed message");
        match self.dispatch.get_mut(&msg.inner.method) {
            Some(m) => {
                let fut = (m)(msg.node_id, msg.inner.body);
                ctx.spawn(wrap_future(async move { fut.await.unwrap(); }));
            }
            None => {
                log::warn!("Message: {} not handled by any global handler", msg.inner.method);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct FromNode<M> {
    pub node_id: Uuid,
    pub inner: M,
}

impl<M> Message for FromNode<M> {
    type Result = Result<(), DispatchError>;
}

pub type NodeHandler = Box<dyn FnMut(Uuid, Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>> + Send>;

pub struct RegisterGlobalHandler {
    method: u32,
    handler: NodeHandler,
}

impl RegisterGlobalHandler {
    /// Create new registration request for node-wide handler
    pub fn new<M>(rec: Recipient<FromNode<M>>) -> Self
    where M: actix::Message<Result=()> + RpcMethod + Send + 'static,
    {
        let handler = move |node_id, data| -> BoxFuture<'static, _> {
            log::info!("Running global message handler");
            let rec = rec.clone();
            Box::pin(async move {
                let msg = M::read(data).map_err(|_| DispatchError::Format)?;
                let msg = FromNode {
                    node_id,
                    inner: msg,
                };
                let res = rec.send(msg).await.map_err(|_| DispatchError::Format)?;
                Ok(Bytes::new())
            })
        };

        RegisterGlobalHandler {
            method: M::ID,
            handler: Box::new(handler),
        }
    }
}

impl Message for RegisterGlobalHandler {
    type Result = ();
}

impl Handler<RegisterGlobalHandler> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: RegisterGlobalHandler, ctx: &mut Context<Self>) -> Self::Result {
        self.dispatch.insert(msg.method, msg.handler);
    }
}
