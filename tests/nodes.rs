use actix::*;
use quix::{self, *};
use quix::process::registry::{ProcessRegistry, Register, Dispatch};
use quix::node::{NodeConfig, NodeControl, Connect};
use actix::clock::Duration;
use uuid::Uuid;
use bytes::Bytes;


#[test]
fn test_nodes() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    std::thread::spawn(|| {
        actix::run(async move {
            NodeConfig::from_registry().send(NodeConfig {
                listen: "127.0.0.1:9001".parse().unwrap(),
                ..Default::default()
            }).await.unwrap();
            NodeControl::from_registry();

            let link = NodeControl::from_registry().send(Connect {
                addr: "127.0.0.1:9002".parse().unwrap()
            }).await.unwrap();

            tokio::time::delay_for(Duration::from_secs(10)).await;
        }).unwrap();
    });

    actix::run(async move {
        NodeConfig::from_registry().send(NodeConfig {
            listen: "127.0.0.1:9002".parse().unwrap(),
            ..Default::default()
        }).await.unwrap();

        let link = NodeControl::from_registry().send(Connect {
            addr: "127.0.0.1:9001".parse().unwrap()
        }).await.unwrap();
        link.send(Dispatch {
            id: Uuid::new_v4(),
            method: "none".to_string(),
            body: Bytes::new()
        }).await.unwrap().unwrap();

        tokio::time::delay_for(Duration::from_secs(10)).await;
    }).unwrap();
}