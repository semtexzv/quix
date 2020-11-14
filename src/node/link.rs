use actix::{Actor, Context, AsyncContext, StreamHandler, Addr, SystemService, ActorFuture, Handler, ActorContext};
use crate::proto::{Meta, Net, Request, Response, PingPong};
use crate::node::{NodeControl, RecvFromNode, NodeConfig, NodeUpdate};
use uuid::Uuid;
use futures::{SinkExt, StreamExt};
use bytes::{Bytes, BytesMut, BufMut, Buf};

use tokio_util::codec::{LengthDelimitedCodec, Encoder, Decoder};
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
use crate::util::{Service, uuid};

pub struct NodeLink {
    id: Uuid,
    correlation_counter: i64,
    stream: FramedWrite<Net, OwnedWriteHalf, NetCodec>,
    running: HashMap<i64, Sender<Result<Bytes, DispatchError>>>,
}

impl Actor for NodeLink {
    type Context = Context<Self>;
}

use std::io;

#[derive(Debug, Copy, Clone)]
pub struct NetCodec;

impl tokio_util::codec::Encoder<Net> for NetCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Net, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let len = prost::Message::encoded_len(&item);
        dst.put_u32(len as _);

        prost::Message::encode(&item, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl tokio_util::codec::Decoder for NetCodec {
    type Item = Net;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        if src.len() < len + 4 {
            return Ok(None);
        }
        let len = src.get_u32() as usize;
        let msg = src.split_to(len);

        prost::Message::decode(msg)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            .map(Some)
    }
}

impl NodeLink {
    pub async fn new(socket: TcpStream) -> (Uuid, SocketAddr, Addr<Self>) {
        let codec = NetCodec;

        socket.set_nodelay(true).unwrap();

        let v = Global::<NodeConfig>::from_registry().send(Get::default()).await.unwrap().unwrap();
        let meta = Meta { nodeid: v.id.as_bytes().to_vec() };

        let peer_addr = socket.peer_addr().unwrap();

        let (rx, tx) = socket.into_split();

        let mut tx = tokio_util::codec::FramedWrite::new(tx, codec);
        let mut rx = tokio_util::codec::FramedRead::new(rx, codec);

        tx.send(Net { meta: Some(meta), ..Default::default() }).await.unwrap();

        let other = rx.next().await.unwrap().unwrap().meta.unwrap();
        let id: Uuid = uuid(other.nodeid.as_slice());

        let tx = tx.into_inner();

        let this = Actor::create(|ctx| {
            ctx.add_stream(rx);

            let tx = FramedWrite::new(tx, NetCodec, ctx);
            ctx.run_interval(Duration::from_secs(1), |this: &mut Self, ctx| {
                log::trace!("Pinging");
                this.stream.write(Net {
                    ping: Some(PingPong {}),
                    ..Default::default()
                });
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

impl StreamHandler<Result<Net, std::io::Error>> for NodeLink {
    fn handle(&mut self, item: Result<Net, Error>, ctx: &mut Context<Self>) {
        let msg = match item {
            Ok(item) => item,
            Err(error) => {
                log::error!("Error occured, disconnecting");
                panic!("Invalid data {:?}", error);
                ctx.stop();
                NodeControl::from_registry().do_send(NodeUpdate::Disconnected(self.id));
                return;
            }
        };

        if let Some(ping) = msg.ping {
            log::trace!("Got pinged");
            self.stream.write(Net {
                pong: Some(ping),
                ..Default::default()
            });
        }
        if let Some(mut req) = msg.request {
            log::trace!("Received request");

            let procid: Uuid = req.procid.map(uuid).unwrap_or(Uuid::nil());

            let dispatch = Dispatch {
                id: procid,
                method: req.methodid,
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
                            let msg = Net {
                                response: Some(Response {
                                    correlation,
                                    body: res.unwrap().unwrap().to_vec(),
                                }),
                                ..Default::default()
                            };
                            this.stream.write(msg);
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
            procid: Some(msg.id.as_bytes().to_vec()),
            methodid: msg.method,
            body: msg.body.to_vec(),
        };

        if msg.wait_for_response {
            self.correlation_counter = self.correlation_counter.wrapping_add(1);

            let (tx, rx) = channel();
            self.running.insert(self.correlation_counter, tx);

            req.correlation = Some(self.correlation_counter);

            let netreq = Net {
                request: Some(req),
                ..Default::default()
            };

            self.stream.write(netreq);
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
            let netreq = Net {
                request: Some(req),
                ..Default::default()
            };
            self.stream.write(netreq);
            // TODO: reply should be an  option here
            actix::Response::reply(Ok(Bytes::new()))
        }
    }
}
