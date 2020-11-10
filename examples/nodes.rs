use actix::{Message, Actor, Handler};
use quix::process::Process;
use derive::ProcessDispatch;

#[derive(Clone, prost::Message)]
pub struct Hello {
    #[prost(string, tag = "1")]
    name: String
}

impl Message for Hello {
    type Result = String;
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