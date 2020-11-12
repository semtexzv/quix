use actix::{Actor, Context, AsyncContext, StreamHandler, Addr, SystemService, ActorFuture, Handler, ActorContext};
use crate::node::proto::{Meta, NodeProtoMessage, Request, Response, PingPong};
use crate::node::{encode, decode, NodeControl, RecvFromNode, NodeConfig, NodeUpdate};
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use bytes::{Bytes, BytesMut};

use tokio_util::codec::{LengthDelimitedCodec};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::io::Error;

use prost::Message;
use actix::io::{WriteHandler, FramedWrite};
use crate::process::registry::{ProcessRegistry, Dispatch};
use actix::fut::wrap_future;
use crate::process::DispatchError;
use std::collections::HashMap;
use tokio::sync::oneshot::{Sender, channel};
use actix::clock::Duration;
use std::net::SocketAddr;
use crate::global::{Global, Get};

pub struct NodeLink {
    id: Uuid,
    correlation_counter: i64,
    stream: FramedWrite<Bytes, OwnedWriteHalf, LengthDelimitedCodec>,
    running: HashMap<i64, Sender<Result<Bytes, DispatchError>>>,
}

impl Actor for NodeLink {
    type Context = Context<Self>;
}

impl NodeLink {
    pub async fn new(socket: TcpStream) -> (Uuid, SocketAddr, Addr<Self>) {
        let codec = tokio_util::codec::LengthDelimitedCodec::builder();

        let v = Global::<NodeConfig>::from_registry().send(Get::default()).await.unwrap().unwrap();
        let x = Meta { id: v.id.to_string() };

        let peer_addr = socket.peer_addr().unwrap();

        let (rx, tx) = socket.into_split();


        let mut tx = codec.new_write(tx);
        let mut rx = codec.new_read(rx);


        tx.send(encode(&x)).await.unwrap();
        let other = rx.next().await.unwrap().unwrap();
        let other: Meta = decode(other).unwrap();
        let id: Uuid = other.id.parse().unwrap();

        let tx = tx.into_inner();

        let this = Actor::create(|ctx| {
            ctx.add_stream(rx);

            let tx = FramedWrite::new(tx, codec.new_codec(), ctx);
            ctx.run_interval(Duration::from_secs(1), |this: &mut Self, ctx| {
                log::trace!("Pinging");
                this.stream.write(encode(&NodeProtoMessage {
                    ping: Some(PingPong {
                        value: "Ping".to_string()
                    }),
                    ..Default::default()
                }));
            });

            NodeLink {
                id: id.clone(),
                correlation_counter: 0,
                stream: tx,
                running: HashMap::new(),
            }
        });
        (id, peer_addr, this)
    }
}

impl WriteHandler<std::io::Error> for NodeLink {}

impl StreamHandler<Result<BytesMut, std::io::Error>> for NodeLink {
    fn handle(&mut self, item: Result<BytesMut, Error>, ctx: &mut Context<Self>) {
        let item = match item {
            Ok(item) => item,
            Err(error) => {
                ctx.stop();
                NodeControl::from_registry().do_send(NodeUpdate::Disconnected(self.id));
                return;
            }
        };

        let msg: NodeProtoMessage = decode(item).unwrap();
        if let Some(ping) = msg.ping {
            log::trace!("Got pinged: {:?}", ping.value);
            self.stream.write(encode(&NodeProtoMessage {
                pong: Some(ping),
                ..Default::default()
            }));
        }
        if let Some(mut req) = msg.request {
            log::trace("Received request");
            let procid: Uuid = req.procid.parse().unwrap();
            let dispatch = Dispatch {
                id: procid,
                method: req.method,
                body: Bytes::from(req.body),
                wait_for_response: req.correlation.is_some(),
            };
            if procid.is_nil() && !dispatch.wait_for_response {
                NodeControl::from_registry().do_send(RecvFromNode {
                    node_id: self.id.clone(),
                    inner: dispatch,
                });
            } else {
                let procreg = ProcessRegistry::from_registry();

                let correlation = req.correlation.take();
                let work = wrap_future(procreg.send(dispatch))
                    .map(move |res, this: &mut Self, ctx| {
                        if let Some(correlation) = correlation {
                            let msg = NodeProtoMessage {
                                response: Some(Response {
                                    correlation,
                                    body: res.unwrap().unwrap().to_vec(),
                                }),
                                ..Default::default()
                            };
                            this.stream.write(encode(&msg));
                        }
                    });
                ctx.spawn(work);
            }
        }

        if let Some(res) = msg.response {
            if let Some(tx) = self.running.remove(&res.correlation) {
                tx.send(Ok(Bytes::from(res.body))).unwrap();
            } else {
                log::error!("Missing correlation id: {}", res.correlation)
            }
        }
    }
}


impl Handler<Dispatch> for NodeLink {
    type Result = actix::Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: Dispatch, ctx: &mut Context<Self>) -> Self::Result {
        let mut req = Request {
            correlation: None,
            procid: msg.id.to_string(),
            method: msg.method,
            body: msg.body.to_vec(),
        };

        if msg.wait_for_response {
            self.correlation_counter = self.correlation_counter.wrapping_add(1);

            let (tx, rx) = channel();
            self.running.insert(self.correlation_counter, tx);

            req.correlation = Some(self.correlation_counter);

            let nm = NodeProtoMessage {
                request: Some(req),
                ..Default::default()
            };

            self.stream.write(encode(&nm));
            actix::Response::fut(Box::pin(async move {
                match tokio::time::timeout(Duration::from_secs(10), rx).await {
                    Ok(Ok(r)) => {
                        r
                    }
                    Ok(Err(e)) => {
                        // closed
                        Err(DispatchError::MailboxRemote)
                    }
                    Err(e) => {
                        // Timeout
                        Err(DispatchError::TimeoutRemote)
                    }
                }
            }))
        } else {
            let nm = NodeProtoMessage {
                request: Some(req),
                ..Default::default()
            };
            self.stream.write(encode(&nm));
            // TODO: reply should be an option here
            actix::Response::reply(Ok(Bytes::new()))
        }
    }
}
