use std::collections::{HashMap, HashSet};
use std::net::{ToSocketAddrs, SocketAddr};
use uuid::{Uuid};
use tokio::stream::StreamExt;
use bytes::{Bytes, BytesMut, Buf};
use futures::sink::SinkExt;

use actix::{Actor, Context, Addr, Handler, ActorFuture, AsyncContext, Message, StreamHandler, ResponseActFuture, SystemService, Supervised, SystemRegistry, System, MessageResult, Recipient};
use actix::fut::{wrap_future, wrap_stream};

mod link;
mod proto;

use crate::node::link::NodeLink;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use futures::{TryStreamExt, Future};
use futures::future::BoxFuture;
use std::pin::Pin;

use crate::util::{RegisterRecipient, Wired};
use crate::global::{Get, Global};
use crate::process::{DispatchError, Dispatcher};
use crate::process::registry::{Dispatch};



/*
pub fn encode<M: prost::Message>(m: &M) -> Bytes {
    let mut buf = BytesMut::new();
    prost::Message::encode(m, &mut buf).unwrap();
    buf.freeze()
}

pub fn decode<B: Buf, M: prost::Message + Default>(buf: B) -> Result<M, prost::DecodeError> {
    prost::Message::decode(buf)
}
*/

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
    pub dispatch: HashMap<&'static str, NodeHandler>,
    pub listeners: HashMap<Uuid, Recipient<NodeUpdate>>,
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
        let item = item.expect("Fatal error in NodeControl");

        let link = wrap_future(NodeLink::new(item));
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

#[derive(Debug, Clone)]
pub enum NodeUpdate {
    Connected(Uuid),
    Disconnected(Uuid),
}

impl Message for NodeUpdate { type Result = (); }

impl Handler<NodeUpdate> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: NodeUpdate, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            NodeUpdate::Disconnected(id) => {
                self.listeners.retain(|_, l| l.do_send(msg.clone()).is_ok());
            }
            _ => panic!("Should receive connected event yet")
        }
    }
}

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

impl Handler<RecvFromNode<Dispatch>> for NodeControl {
    type Result = ();

    fn handle(&mut self, msg: RecvFromNode<Dispatch>, ctx: &mut Self::Context) -> Self::Result {
        log::info!("NodeControl dispatching unadressed message");
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
pub struct RecvFromNode<M> {
    pub node_id: Uuid,
    pub inner: M,
}

impl<M> Message for RecvFromNode<M> {
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
    pub fn new<M>(rec: Recipient<RecvFromNode<M>>) -> Self
    where M: actix::Message<Result=()> + Wired + Send + 'static,
    {
        let handler = move |node_id, data| -> Pin<Box<dyn Future<Output=_>>> {
            log::info!("Running global message handler");
            let rec = rec.clone();
            Box::pin(async move {
                let msg = M::read(data).map_err(|_| DispatchError::Format)?;
                let msg = RecvFromNode {
                    node_id,
                    inner: msg,
                };
                let res = rec.send(msg).await.map_err(|_| DispatchError::Format)?;
                Ok(Bytes::new())
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
