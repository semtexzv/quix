use crate::import::*;
use crate::proto::{Get, Value};
use crate::Process;
use crate::process::DispatchError;
use crate::node::{NodeControl, RegisterGlobalHandler, FromNode};

pub struct Write {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl Message for Write { type Result = (); }

impl Write {
    pub fn new<MK: prost::Message, MV: prost::Message>(k: MK, v: MV) -> Self {
        let mut res = Write {
            key: vec![],
            value: vec![],
        };

        k.encode(&mut res.key).unwrap();
        v.encode(&mut res.value).unwrap();
        res
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
        NodeControl::from_registry().do_send(RegisterGlobalHandler::new::<Get, _>(ctx.address().recipient()));
    }
}

impl Supervised for MemKv {}

impl SystemService for MemKv {}

impl Handler<Write> for MemKv {
    type Result = ();

    fn handle(&mut self, msg: Write, ctx: &mut Context<Self>) -> Self::Result {
        self.data.insert(msg.key, msg.value);
    }
}

impl Handler<Get> for MemKv {
    type Result = Result<Value, DispatchError>;

    fn handle(&mut self, msg: Get, ctx: &mut Self::Context) -> Self::Result {
        let data = self.data.get(&msg.data);
        let res = Value {
            data: data.cloned()
        };
        Ok(res)
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