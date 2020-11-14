use actix::{Message, Actor, Handler};
use quix::process::Process;
use derive::ProcessDispatch;
use quix::util::Service;
use bytes::{Buf, BufMut};

#[derive(Clone, prost::Message)]
pub struct Hello {
    #[prost(string, tag = "1")]
    name: String
}

impl Message for Hello {
    type Result = String;
}

impl Service for Hello {
    const NAME: &'static str = "";
    const ID: u64 = 42;

    fn read(b: impl Buf) -> Result<Self, ()> {
        unimplemented!()
    }

    fn write(&self, b: &mut impl BufMut) -> Result<(), ()> {
        unimplemented!()
    }

    fn read_result(b: impl Buf) -> Result<Self::Result, ()> {
        unimplemented!()
    }

    fn write_result(r: &Self::Result, b: &mut impl BufMut) -> Result<(), ()> {
        unimplemented!()
    }
}

#[derive(ProcessDispatch)]
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