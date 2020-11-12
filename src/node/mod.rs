use std::collections::{HashMap, HashSet};
use std::net::{ToSocketAddrs, SocketAddr};
use uuid::{Uuid};
use tokio::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf};
use futures::sink::SinkExt;

use actix::{Actor, Context, Addr, Handler, ActorFuture, AsyncContext, Message, StreamHandler, ResponseActFuture, SystemService, Supervised, SystemRegistry, System, MessageResult, Recipient};
use actix::fut::{wrap_future, wrap_stream};

mod link;

use crate::node::link::NodeLink;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use futures::{TryStreamExt, Future};
use crate::process::{DispatchError, Dispatcher};
use crate::process::registry::{Dispatch};
use futures::future::BoxFuture;
use std::pin::Pin;
use crate::util::RegisterRecipient;

/// Messages exchanged between individual nodes on lowest protocol level
#[derive(prost::Message)]
pub struct NodeProtoMessage {
    #[prost(message, optional, tag = "1")]
    ping: Option<PingPong>,

    #[prost(message, optional, tag = "2")]
    pong: Option<PingPong>,

    #[prost(message, optional, tag = "3")]
    meta: Option<Meta>,

    #[prost(message, optional, tag = "4")]
    request: Option<Request>,

    #[prost(message, optional, tag = "5")]
    response: Option<Response>,

    /*
    #[prost(message, optional, tag = "6")]
    announce: Option<Announce>,
     */
}

#[derive(prost::Message)]
pub struct PingPong {
    #[prost(string, tag = "1")]
    value: String
}

#[derive(prost::Message)]
pub struct Meta {
    #[prost(string, tag = "1")]
    id: String
}

#[derive(prost::Message)]
pub struct Request {
    #[prost(string, required, tag = "1")]
    procid: String,

    #[prost(string, required, tag = "2")]
    method: String,

    #[prost(int64, optional, tag = "3")]
    correlation: Option<i64>,

    #[prost(bytes, required, tag = "4")]
    body: Vec<u8>,
}

#[derive(prost::Message)]
pub struct Response {
    #[prost(int64, required, tag = "1")]
    correlation: i64,

    #[prost(bytes, required, tag = "2")]
    body: Vec<u8>,
}

#[derive(prost::Message)]
pub struct NodeInfo {
    #[prost(string, required, tag = "1")]
    id: String,

    #[prost(string, required, tag = "2")]
    socket_addr: String,
}

/*
#[derive(prost::Message)]
pub struct Announce {
    #[prost(string, repeated, tag = "1")]
    newprocs: Vec<String>,

    #[prost(string, repeated, tag = "2")]
    delprocs: Vec<String>,

    #[prost(message, repeated, tag = "3")]
    nodes: Vec<NodeInfo>,

    #[prost(map = "string, string", required, tag = "4")]
    handlers: HashMap<String, String>,
}
 */

pub fn encode<M: prost::Message>(m: &M) -> Bytes {
    let mut buf = BytesMut::new();
    prost::Message::encode(m, &mut buf).unwrap();
    buf.freeze()
}

pub fn decode<B: Buf, M: prost::Message + Default>(buf: B) -> Result<M, prost::DecodeError> {
    prost::Message::decode(buf)
}

pub struct GetConfig;

impl Message for GetConfig {
    type Result = NodeConfig;
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: Uuid,
    pub listen: SocketAddr,
}

impl Message for NodeConfig {
    type Result = ();
}

impl Supervised for NodeConfig {}

impl SystemService for NodeConfig {}

impl Actor for NodeConfig {
    type Context = Context<Self>;
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self { listen: "127.0.0.1:9000".parse().unwrap(), id: Uuid::new_v4() }
    }
}

impl Handler<NodeConfig> for NodeConfig {
    type Result = ();

    fn handle(&mut self, msg: NodeConfig, ctx: &mut Self::Context) -> Self::Result {
        *self = msg;
    }
}

impl Handler<GetConfig> for NodeConfig {
    type Result = MessageResult<GetConfig>;

    fn handle(&mut self, msg: GetConfig, ctx: &mut Self::Context) -> Self::Result {
        return MessageResult(self.clone());
    }
}

pub struct NodeControl {
    /// Links to other nodes
    links: HashMap<Uuid, Addr<NodeLink>>,
    /// Dispatcher for unaddressed messages.
    ///
    /// Messages which are sent to process id of 00000000000000000....
    /// are considered unadressed, and are dispatched from here
    pub dispatch: HashMap<&'static str, NodeHandler>,
    pub listeners: HashMap<Uuid, Recipient<NodeUpdate>>,
}

impl SystemService for NodeControl {}

impl Supervised for NodeControl {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        let cfg = NodeConfig::from_registry().send(GetConfig);
        let set_cfg = wrap_future(cfg)
            .then(|cfg, this: &mut Self, ctx| {
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
            listeners: HashMap::new(),
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
        let link = wrap_future(NodeLink::new(item.unwrap()));
        let fut = link
            .map(|(id, peer, link), this: &mut Self, ctx| {
                log::info!("Connected to: {:?}", id);
                this.links.insert(id, link.clone());
                this.listeners.retain(|_, l| {
                    l.do_send(NodeUpdate::Connected(id)).is_ok()
                });
            });
        ctx.spawn(fut);
    }
}

#[derive(Debug)]
pub enum NodeUpdate {
    Connected(Uuid),
    Disconnected(Uuid),
}

impl Message for NodeUpdate { type Result = (); }

impl Handler<RegisterRecipient<NodeUpdate>> for NodeControl {
    type Result = Result<Uuid, std::convert::Infallible>;

    fn handle(&mut self, msg: RegisterRecipient<NodeUpdate>, ctx: &mut Self::Context) -> Self::Result {
        let id = Uuid::new_v4();
        self.listeners.insert(id, msg.0);
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

pub struct SendToNode(pub Uuid, pub Dispatch);

impl Message for SendToNode {
    type Result = Result<Bytes, DispatchError>;
}

impl Handler<SendToNode> for NodeControl {
    type Result = actix::Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: SendToNode, ctx: &mut Self::Context) -> Self::Result {
        if let Some(link) = self.links.get(&msg.0) {
            let link = link.clone();
            let work = async move {
                link.send(msg.1).await.unwrap()
            };
            actix::Response::fut(Box::pin(work))
        } else {
            return actix::Response::reply(Err(DispatchError::DispatchRemote));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Broadcast(pub Dispatch);

impl Message for Broadcast { type Result = (); }

impl Handler<Broadcast> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) -> Self::Result {
        for n in self.links.values() {
            n.do_send(msg.0.clone());
        }
    }
}

impl Handler<FromRemote<Dispatch>> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: FromRemote<Dispatch>, ctx: &mut Self::Context) -> Self::Result {
        log::info!("NodeControl dispatching");
        match self.dispatch.get_mut(msg.inner.method.as_str()) {
            Some(m) => {
                let fut = (m)(msg.node_id, msg.inner.body);
                ctx.spawn(wrap_future(async move { fut.await.unwrap(); }));
            }
            None => {
                log::warn!("Message: {} not handled by any global handler", msg.inner.method);
            }
        }
    }
}

#[derive(Debug)]
pub struct FromRemote<M> {
    pub node_id: Uuid,
    pub inner: M,
}

impl<M> Message for FromRemote<M> {
    type Result = ();
}

pub type NodeHandler = Box<dyn FnMut(Uuid, Bytes)
    -> Pin<Box<dyn Future<Output=Result<Bytes, DispatchError>>>> + Send + 'static>;

pub struct RegisterSystemHandler {
    method: &'static str,
    handler: NodeHandler,
}

impl RegisterSystemHandler {
    /// Create new registration request for node-wide handler
    pub fn new<M>(rec: Recipient<FromRemote<M>>) -> Self
    where M: actix::Message<Result=()> + prost::Message + Send + Default + 'static,
    {
        let handler = move |node_id, data| -> Pin<Box<dyn Future<Output=_>>> {
            log::info!("Running global message handler");
            let rec = rec.clone();
            Box::pin(async move {
                let msg = M::decode(data).map_err(|_| DispatchError::Format)?;
                let msg = FromRemote {
                    node_id,
                    inner: msg,
                };
                let res = rec.send(msg).await.map_err(|_| DispatchError::Format)?;
                Ok(encode(&res))
            })
        };

        RegisterSystemHandler {
            method: core::any::type_name::<M>(),
            handler: Box::new(handler),
        }
    }
}

impl Message for RegisterSystemHandler {
    type Result = ();
}

impl Handler<RegisterSystemHandler> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: RegisterSystemHandler, ctx: &mut Context<Self>) -> Self::Result {
        self.dispatch.insert(msg.method, msg.handler);
    }
}
