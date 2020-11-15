use actix::*;
use quix::{self, *};
use quix::process::registry::{ProcessRegistry, Register};
use quix::util::RpcMethod;
use bytes::{Buf, BufMut};
use quix::process::DispatchError;


#[derive(prost::Message)]
pub struct M {
    #[prost(int32, tag = "1")]
    v: i32
}
impl Message for M {
    type Result = i32;
}
impl RpcMethod for M {
    const NAME: &'static str = "M";
    const ID: u32 = 42;

    fn read(b: impl Buf) -> Result<Self,DispatchError> {
        unimplemented!()
    }

    fn write(&self, b: &mut impl BufMut) -> Result<(), DispatchError> {
        unimplemented!()
    }

    fn read_result(b: impl Buf) -> Result<Self::Result, DispatchError> {
        unimplemented!()
    }

    fn write_result(r: &Self::Result, b: &mut impl BufMut) -> Result<(), DispatchError> {
        unimplemented!()
    }
}

#[derive(quix::DynHandler)]
#[dispatch(M)]
pub struct Act {}

impl Actor for Act {
    type Context = Process<Self>;
}

impl Handler<M> for Act {
    type Result = i32;

    fn handle(&mut self, _msg: M, _ctx: &mut Process<Self>) -> Self::Result {
        unimplemented!()
    }
}

#[test]
fn test_register_process() {
    actix::run(async move {
        let procs = ProcessRegistry::from_registry();
        let a = Process::start(Act {});
        let a = procs.send(Register::new(a.clone())).await.unwrap();
    }).unwrap();
}
