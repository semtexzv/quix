use actix::{Actor, Context, AsyncContext, StreamHandler, Addr, SystemService, ActorFuture, Handler};
use crate::node::{Meta, encode, decode, NodeMessage, Response, Request, PingPong, NodeConfig, GetConfig};
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

pub struct NodeLink {
    id: i64,
    stream: FramedWrite<Bytes, OwnedWriteHalf, LengthDelimitedCodec>,
    running: HashMap<i64, Sender<Result<Bytes, DispatchError>>>,
}

impl Actor for NodeLink {
    type Context = Context<Self>;
}

impl NodeLink {
    pub async fn new(socket: TcpStream) -> (Uuid, Addr<Self>) {
        let codec = tokio_util::codec::LengthDelimitedCodec::builder();

        let v = NodeConfig::from_registry().send(GetConfig).await.unwrap();
        let x = Meta { id: v.id.to_string() };

        let (rx, tx) = socket.into_split();

        let mut tx = codec.new_write(tx);
        let mut rx = codec.new_read(rx);


        tx.send(encode(&x)).await.unwrap();
        let other = rx.next().await.unwrap().unwrap();
        let other: Meta = decode(other).unwrap();

        let tx = tx.into_inner();

        let this = Actor::create(|ctx| {
            ctx.add_stream(rx);

            let tx = FramedWrite::new(tx, codec.new_codec(), ctx);
            ctx.run_interval(Duration::from_secs(1), |this: &mut Self, ctx| {
                log::trace!("Pinging");
                this.stream.write(encode(&NodeMessage {
                    ping: Some(PingPong {
                        value: "Ping".to_string()
                    }),
                    ..Default::default()
                }));
            });

            NodeLink {
                id: 0,
                stream: tx,
                running: HashMap::new(),
            }
        });
        (other.id.parse().unwrap(), this)
    }
}

impl WriteHandler<std::io::Error> for NodeLink {}

impl StreamHandler<Result<BytesMut, std::io::Error>> for NodeLink {
    fn handle(&mut self, item: Result<BytesMut, Error>, ctx: &mut Context<Self>) {
        let msg: NodeMessage = decode(item.unwrap()).unwrap();
        if let Some(ping) = msg.ping {
            log::trace!("Got pinged: {:?}", ping.value);
            self.stream.write(encode(&NodeMessage {
                pong: Some(ping),
                ..Default::default()
            }));
        }
        if let Some(mut req) = msg.request {
            let procid: Uuid = req.procid.parse().unwrap();
            let procreg = ProcessRegistry::from_registry();
            let dispatch = Dispatch {
                id: procid,
                method: req.method,
                body: Bytes::from(req.body),
            };
            let correlation = req.correlation.take();
            let work = wrap_future(procreg.send(dispatch))
                .map(move |res, this: &mut Self, ctx| {
                    if let Some(correlation) = correlation {
                        let msg = NodeMessage {
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
        self.id = self.id.wrapping_add(1);

        let (tx, rx) = channel();
        self.running.insert(self.id, tx);

        let req = NodeMessage {
            request: Some(Request {
                correlation: Some(self.id),
                procid: msg.id.to_string(),
                method: msg.method,
                body: msg.body.to_vec(),
            }),
            ..Default::default()
        };

        self.stream.write(encode(&req));
        return actix::Response::fut(Box::pin(async move { rx.await.unwrap() }));
    }
}