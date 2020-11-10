use std::collections::HashMap;
use std::net::{ToSocketAddrs, SocketAddr};
use uuid::{Uuid};
use tokio::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf};
use futures::sink::SinkExt;

use actix::{Actor, Context, Addr, Handler, ActorFuture, AsyncContext, Message, StreamHandler, ResponseActFuture, SystemService, Supervised, SystemRegistry, System, MessageResult};
use actix::fut::{wrap_future, wrap_stream};

mod link;

use crate::node::link::NodeLink;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use futures::TryStreamExt;
use crate::process::DispatchError;

/// Messages exchanged between individual nodes
#[derive(prost::Message)]
pub struct NodeMessage {
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
pub struct Announce {
    #[prost(string, required, tag = "1")]
    name: String,

    #[prost(string, required, tag = "1")]
    procid: String,
}

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
    pub listen: SocketAddr
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
        Self { listen: "127.0.0.1:9000".parse().unwrap() }
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
    links: HashMap<Uuid, Addr<NodeLink>>,
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
            .map(|(id, link), this: &mut Self, ctx| {
                this.links.insert(id, link.clone());
            });
        ctx.spawn(fut);
    }
}


pub struct Listen {
    pub(crate) listen_addr: SocketAddr
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

        let conn = async move {
            let stream = TcpStream::connect(msg.addr).await.unwrap();
            NodeLink::new(stream).await
        };

        let link = wrap_future(conn);
        Box::pin(link.map(|(id, link), this: &mut Self, ctx| {
            this.links.insert(id, link.clone());
            link
        }))
    }
}

