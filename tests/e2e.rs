use quix::node::{NodeConfig, NodeController, Connect, NodeId};
use actix::{SystemService, Message, Actor, Handler, Running};
use std::time::Duration;
use quix::{Process};
use std::thread::JoinHandle;
use quix::global::{Global, Set};
use quix::util::RpcMethod;
use bytes::{Buf, BufMut};
use quix::process::DispatchError;

use quix::proto::*;

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

    fn read(b: impl Buf) -> Result<Self, DispatchError> {
        unimplemented!()
    }

    fn write(&self, b: &mut impl BufMut) -> Result<(), DispatchError> {
        unimplemented!()
    }

    fn read_result(b: impl Buf) -> Self::Result {
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

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::warn!("Stopping actor");
        Running::Stop
    }
}

impl Handler<M> for Act {
    type Result = i32;

    fn handle(&mut self, _msg: M, _ctx: &mut Process<Self>) -> Self::Result {
        unimplemented!()
    }
}


fn make_node(i: i32) -> JoinHandle<()> {
    std::thread::spawn(move || {
        actix::run(async move {
            tokio::time::delay_for(Duration::from_millis((i * 100) as u64)).await;

            let config = NodeConfig {
                listen: format!("127.0.0.1:900{}", i).parse().unwrap(),
                ..Default::default()
            };

            Global::<NodeConfig>::from_registry().send(Set(config)).await.unwrap();

            if i > 0 {
                let _ = NodeController::from_registry().send(Connect {
                    addr: format!("127.0.0.1:900{}", i - 1).parse().unwrap()
                }).await.unwrap();
            }

            let _ = Process::start(Act {});
            // m2 should be deleted after the end of the block
            {
                let _ = Process::start(Act {});
                tokio::time::delay_for(Duration::from_secs(1)).await;
            }

            tokio::time::delay_for(Duration::from_secs(10)).await;
        }).unwrap();
    })
}

#[test]
fn test_e2e() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let n0 = make_node(0);
    let n1 = make_node(1);
    let n2 = make_node(2);

    n0.join().unwrap();
    n1.join().unwrap();
    n2.join().unwrap();
}