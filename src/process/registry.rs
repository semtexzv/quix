use std::collections::HashMap;
use uuid::Uuid;
use actix::{Recipient, Actor, Context, Supervised, SystemService, Handler, Message, Response};
use crate::process::{Dispatcher, ProcessDispatch, Pid, Process, DispatchError};
use bytes::Bytes;

pub struct ProcessRegistry {
    processes: HashMap<Uuid, Box<dyn Dispatcher>>
}

impl Actor for ProcessRegistry {
    type Context = Context<Self>;
}

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self {
            processes: HashMap::new()
        }
    }
}

impl Supervised for ProcessRegistry {}

impl SystemService for ProcessRegistry {}


pub struct Register {
    id: Uuid,
    dispatcher: Box<dyn Dispatcher>,
}

impl Register {
    pub fn new<A: ProcessDispatch>(pid: Pid<A>) -> Self {
        let dispatcher = A::make_dispatcher(pid.local_addr().unwrap());
        let id = pid.id();
        Register {
            id,
            dispatcher,
        }
    }
}

impl Message for Register {
    type Result = ();
}

impl Handler<Register> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: Register, ctx: &mut Context<Self>) -> Self::Result {
        let _ = self.processes.insert(msg.id, msg.dispatcher);
    }
}

pub struct Unregister {
    id: Uuid
}

impl Message for Unregister {
    type Result = ();
}

impl Handler<Unregister> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: Unregister, ctx: &mut Context<Self>) -> Self::Result {
        self.processes.remove(&msg.id);
    }
}


pub struct Dispatch {
    pub(crate) id: Uuid,
    pub(crate) method: String,
    pub(crate) body: Bytes,
}

impl Message for Dispatch {
    type Result = Result<Bytes, DispatchError>;
}

impl Handler<Dispatch> for ProcessRegistry {
    type Result = Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: Dispatch, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(p) = self.processes.get(&msg.id) {
            Response::fut(p.dispatch(msg.method, msg.body))
        } else {
            Response::reply(Err(DispatchError {}))
        }
    }
}