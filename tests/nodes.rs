use actix::*;
use quix::{self};
use quix::node::{NodeConfig, NodeControl, Connect};
use actix::clock::Duration;
use quix::global::{Global, Set};

#[test]
fn test_nodes() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    std::thread::spawn(|| {
        actix::run(async move {
            Global::<NodeConfig>::from_registry().send(Set(NodeConfig {
                listen: "127.0.0.1:9001".parse().unwrap(),
                ..Default::default()
            })).await.unwrap();
            NodeControl::from_registry();

            let link = NodeControl::from_registry().send(Connect {
                addr: "127.0.0.1:9002".parse().unwrap()
            }).await.unwrap();

            tokio::time::delay_for(Duration::from_secs(10)).await;
        }).unwrap();
    });

    actix::run(async move {
        Global::<NodeConfig>::from_registry().send(Set(NodeConfig {
            listen: "127.0.0.1:9002".parse().unwrap(),
            ..Default::default()
        })).await.unwrap();

        let link = NodeControl::from_registry().send(Connect {
            addr: "127.0.0.1:9001".parse().unwrap()
        }).await.unwrap();

        tokio::time::delay_for(Duration::from_secs(10)).await;
    }).unwrap();
}