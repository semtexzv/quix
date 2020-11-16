use crate::{
    import::*,
    Broadcast,
    node::NodeController,
    node::FromNode,
    node::NodeConfig,
    node::NodeStatus,
    proto::Meta,
    proto::Net,
    proto::Request,
    proto::Response,
    proto::PingPong,
    process::registry::ProcessRegistry,
    global::Global,
    global::Get,
    util::RpcMethod,
    util::uuid,
    MethodCall,
    ProcDispatch,
};

use std::io;
use actix::io::{FramedWrite, WriteHandler};
use tokio::{
    net::tcp::OwnedWriteHalf,
    net::TcpStream,
};
use actix::Running;
use futures::io::Error;
use crate::process::DispatchError;


pub struct NodeLink {
    id: Uuid,
    correlation_counter: i64,
    stream: FramedWrite<Net, OwnedWriteHalf, NetCodec>,
    running: HashMap<i64, Sender<Result<Bytes, DispatchError>>>,
}

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
    /// Open connection to a node, exchange IDs, setup local state
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

    fn handle_return_correlation(&mut self, ctx: &mut Context<Self>, res: Result<Bytes, DispatchError>, corr: i64) {
        let (ok, err) = match res {
            Ok(v) => (Some(v.to_vec()), None),
            Err(e) => (None, Some(e.code())),
        };

        let res = Response {
            correlation: corr,
            body: ok,
            error: err,
        };
        let msg = Net {
            response: Some(res),
            ..Default::default()
        };
        self.stream.write(msg);
    }

    fn handle_request(&mut self, ctx: &mut Context<Self>, req: Request) {
        log::trace!("Received request");

        let procid: Option<Uuid> = req.procid.map(uuid).filter(|v| !v.is_nil());

        let dispatch = MethodCall {
            method: req.methodid,
            body: Bytes::from(req.body),
        };
        if let Some(procid) = procid {
            let procreg = ProcessRegistry::from_registry();

            let dispatch = ProcDispatch {
                procid,
                inner: dispatch,
            };

            if let Some(corr) = req.correlation {
                let work = wrap_future(procreg.send(dispatch).map(|r| r.unwrap()));
                let work = work.map(move |res, this: &mut Self, ctx| this.handle_return_correlation(ctx, res, corr));
                ctx.spawn(work);
            } else {
                procreg.do_send(dispatch);
            }
        } else {
            let nodecontrol = NodeController::from_registry();

            let dispatch = FromNode {
                node_id: self.id,
                inner: dispatch,
            };

            if let Some(corr) = req.correlation {
                let work = wrap_future(nodecontrol.send(dispatch).map(|r| r.unwrap()));
                let work = work.map(move |res, this: &mut Self, ctx| this.handle_return_correlation(ctx, res, corr));
                ctx.spawn(work);
            } else {
                // Err, here it should be a broadcast
                //ctx.spawn(wrap_future(async move { nodecontrol.send(dispatch).await.unwrap().unwrap(); }));
                nodecontrol.do_send(dispatch);
                // Without process ID, we currently only handle notifications
            }
        }
    }
}

impl Actor for NodeLink {
    type Context = Context<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        NodeController::from_registry().do_send(NodeStatus::Disconnected(self.id));
    }
}

impl WriteHandler<std::io::Error> for NodeLink {
    fn error(&mut self, err: Error, ctx: &mut Self::Context) -> Running {
        panic!("Invalid data {:?}", err);
        // Stop on errors
        Running::Stop
    }
}

impl StreamHandler<io::Result<Net>> for NodeLink {
    fn handle(&mut self, item: io::Result<Net>, ctx: &mut Context<Self>) {
        let msg = match item {
            Ok(item) => item,
            Err(error) => {
                log::error!("Error occured, disconnecting");
                panic!("Invalid data {:?}", error);
                ctx.stop();
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
            self.handle_request(ctx, req);
        }

        if let Some(res) = msg.response {
            if let Some(tx) = self.running.remove(&res.correlation) {
                if let Some(err) = res.error {
                    // TODO: Error from i32
                    let _ = tx.send(Err(DispatchError::from_code(err)));
                } else if let Some(body) = res.body {
                    let _ = tx.send(Ok(Bytes::from(body)));
                } else {
                    log::error!("Received response without error or body");
                    let _ = tx.send(Err(DispatchError::Protocol));
                }
            } else {
                log::error!("Missing correlation id: {}", res.correlation)
            }
        }
    }
}

impl Handler<Broadcast> for NodeLink {
    type Result = Result<(), DispatchError>;

    fn handle(&mut self, msg: Broadcast, ctx: &mut Self::Context) -> Self::Result {
        let mut req = Request {
            correlation: None,
            procid: None,

            methodid: msg.method,
            body: msg.body.to_vec(),
        };

        let netreq = Net {
            request: Some(req),
            ..Default::default()
        };

        self.stream.write(netreq);
        Ok(())
    }
}

impl Handler<MethodCall> for NodeLink {
    type Result = actix::Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: MethodCall, ctx: &mut Context<Self>) -> Self::Result {
        let mut req = Request {
            correlation: None,
            procid: None,

            methodid: msg.method,
            body: msg.body.to_vec(),
        };

        self.correlation_counter = self.correlation_counter.wrapping_add(1);

        let (tx, rx) = futures::channel::oneshot::channel();
        self.running.insert(self.correlation_counter, tx);

        req.correlation = Some(self.correlation_counter);

        let net = Net {
            request: Some(req),
            ..Default::default()
        };

        self.stream.write(net);
        // Safety timeout
        let fut = tokio::time::timeout(Duration::from_secs(60), rx);
        actix::Response::fut(fut
            .map_err(|_| DispatchError::Timeout)
            .map(|v| v.map(|v| v.map_err(|_| DispatchError::MailboxRemote))??)
        )
    }
}
