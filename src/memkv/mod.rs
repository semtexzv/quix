use crate::import::*;
use crate::proto::{Get, Value, Key};
use crate::{Process, NodeDispatch};
use crate::process::DispatchError;
use crate::node::{NodeController, RegisterGlobalHandler, FromNode, NodeId, ListNodes};
use crate::util::RpcMethod;
use futures::TryStreamExt;
use futures::stream::FuturesUnordered;

pub struct Write {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
}

impl Message for Write { type Result = (); }

pub struct LocalRead {
    key: Vec<u8>,
}

impl Message for LocalRead { type Result = Option<Vec<u8>>; }

pub struct RemoteRead {
    node: Uuid,
    key: Vec<u8>,
}

impl Message for RemoteRead {
    type Result = Result<Option<Vec<u8>>, DispatchError>;
}

pub struct GlobalFind {
    pub key: Vec<u8>,
}

impl Message for GlobalFind {
    type Result = Result<Option<Vec<u8>>, DispatchError>;
}


impl Handler<Write> for MemKv {
    type Result = ();

    fn handle(&mut self, msg: Write, ctx: &mut Context<Self>) -> Self::Result {
        self.data.insert(msg.key, msg.value);
    }
}

impl Handler<LocalRead> for MemKv {
    type Result = Option<Vec<u8>>;

    fn handle(&mut self, msg: LocalRead, ctx: &mut Self::Context) -> Self::Result {
        self.data.get(&msg.key).cloned()
    }
}

impl Handler<RemoteRead> for MemKv {
    type Result = ResponseFuture<Result<Option<Vec<u8>>, DispatchError>>;

    fn handle(&mut self, msg: RemoteRead, ctx: &mut Context<Self>) -> Self::Result {
        Box::pin(NodeId(msg.node).send(Get(Key { data: msg.key })).map_ok(|v| v.data))
    }
}

impl Handler<GlobalFind> for MemKv {
    type Result = ResponseFuture<Result<Option<Vec<u8>>, DispatchError>>;

    fn handle(&mut self, msg: GlobalFind, ctx: &mut Context<Self>) -> Self::Result {
        let nc = NodeController::from_registry();
        let nodes = nc.send(ListNodes);

        let uuids = nodes.map(|r| r.unwrap());
        let res = async move {
            let mut tasks: FuturesUnordered<_> = uuids.await.into_iter().map(|n| {
                NodeId(n).send(Get(Key { data: msg.key.clone() })).map_ok(|v| v.data)
            }).collect();
            // We have received A response ( one future resolved, which had an OK result, and the result vas an actual value)
            while let Some(Ok(Some(data))) = tasks.next().await {
                return Ok(Some(data));
            }
            return Ok(None);
        };
        Box::pin(res)
    }
}

/// Simple in-memory key-value store.
/// Local instance can be modified, remote instances can be only read
#[derive(Default)]
pub struct MemKv {
    data: BTreeMap<Vec<u8>, Vec<u8>>
}

impl Actor for MemKv {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        NodeController::from_registry().do_send(RegisterGlobalHandler::new::<Get, _>(ctx.address().recipient()));
    }
}

impl Supervised for MemKv {}

impl SystemService for MemKv {}


impl Handler<Get> for MemKv {
    type Result = Result<crate::proto::Value, DispatchError>;

    fn handle(&mut self, msg: Get, ctx: &mut Self::Context) -> Self::Result {
        let res = self.data.get(&msg.data).map(|v| v.to_vec());
        Ok(crate::proto::Value {
            data: res
        })
    }
}

impl Handler<FromNode<Get>> for MemKv {
    type Result = Result<crate::proto::Value, DispatchError>;

    fn handle(&mut self, msg: FromNode<Get>, ctx: &mut Self::Context) -> Self::Result {
        let res = self.data.get(&msg.inner.0.data).map(|v| v.to_vec());
        Ok(crate::proto::Value {
            data: res
        })
    }
}