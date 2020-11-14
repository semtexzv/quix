use crate::import::*;
use crate::proto::{Get, Value};
use crate::Process;


pub struct Put {
    key: Vec<u8>,
    value: Vec<u8>,
}

impl Message for Put { type Result = (); }

/// Simple in-memory key-value store.
/// Local instance can be modified, remote instances can be only read
#[derive(Default)]
pub struct MemKv {
    data: BTreeMap<Vec<u8>, Vec<u8>>
}

impl Actor for MemKv {
    type Context = Context<Self>;
}

impl Supervised for MemKv {

}

impl SystemService for MemKv {}

impl Handler<Put> for MemKv {
    type Result = ();

    fn handle(&mut self, msg: Put, ctx: &mut Context<Self>) -> Self::Result {
        self.data.insert(msg.key, msg.value);
    }
}

impl Handler<Get> for MemKv {
    type Result = Result<Value, ()>;

    fn handle(&mut self, msg: Get, ctx: &mut Self::Context) -> Self::Result {
        let data = self.data.get(&msg.data);
        let res = Value {
            data: data.cloned()
        };
        Ok(res)
    }
}