use actix::{Message, Actor, Handler};
use quix::process::{Process, DispatchError};
use derive::DynHandler;
use quix::util::RpcMethod;
use bytes::{Buf, BufMut};

#[derive(Clone, prost::Message)]
pub struct Hello {
    #[prost(string, tag = "1")]
    name: String
}

impl Message for Hello {
    type Result = String;
}

impl RpcMethod for Hello {
    const NAME: &'static str = "";
    const ID: u32 = 42;

    fn read(_: impl Buf) -> Result<Self, DispatchError> {
        unimplemented!()
    }

    fn write(&self, _: &mut impl BufMut) -> Result<(), DispatchError> {
        unimplemented!()
    }

    fn read_result(_: impl Buf) -> Self::Result {
        unimplemented!()
    }

    fn write_result(_: &Self::Result, _b: &mut impl BufMut) -> Result<(), DispatchError> {
        unimplemented!()
    }
}

#[derive(DynHandler)]
#[dispatch(Hello)]
pub struct World {
    hellos: Vec<String>
}

impl Actor for World {
    type Context = Process<Self>;
}

impl Handler<Hello> for World {
    type Result = String;

    fn handle(&mut self, msg: Hello, _ctx: &mut Process<Self>) -> Self::Result {
        self.hellos.push(msg.name);
        return "World".to_string();
    }
}

fn main() {
    env_logger::init();
}