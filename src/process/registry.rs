use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use actix::{Recipient, Actor, Context, Supervised, SystemService, Handler, Message, Response, AsyncContext, ContextFutureSpawner};
use crate::process::{Dispatcher, ProcessDispatch, Pid, Process, DispatchError};
use bytes::Bytes;
use crate::node::{NodeControl, SendToNode, RegisterSystemHandler, RecvFromNode, Broadcast, NodeUpdate};
use futures::FutureExt;
use actix::clock::Duration;
use actix::fut::wrap_future;
use std::str::FromStr;
use crate::derive::ProstMessage;
use crate::util::{RegisterRecipient, Wired};

pub struct ProcessRegistry {
    local: HashMap<Uuid, Box<dyn Dispatcher>>,
    nodes: HashMap<Uuid, Uuid>,

    new: HashSet<Uuid>,
    deleted: HashSet<Uuid>,
}

impl Actor for ProcessRegistry {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        Supervised::restarting(self, ctx);
    }
}

impl Default for ProcessRegistry {
    fn default() -> Self {
        Self {
            local: HashMap::new(),
            nodes: HashMap::new(),

            new: HashSet::new(),
            deleted: HashSet::new(),
        }
    }
}

impl SystemService for ProcessRegistry {}

#[derive(prost::Message)]
pub struct ProcessList {
    #[prost(string, repeated, tag = "2")]
    new: Vec<String>,
    #[prost(string, repeated, tag = "3")]
    del: Vec<String>,
}

impl actix::Message for ProcessList { type Result = (); }

impl Supervised for ProcessRegistry {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        log::info!("Setting up process registry");
        let control = NodeControl::from_registry();

        control.do_send(RegisterRecipient(ctx.address().recipient::<NodeUpdate>()));
        control.do_send(RegisterSystemHandler::new::<ProcessList>(ctx.address().recipient()));

        ctx.run_interval(Duration::from_millis(800), |this, ctx| {
            log::info!("Sending process table update");
            let new = std::mem::replace(&mut this.new, HashSet::new());
            let del = std::mem::replace(&mut this.deleted, HashSet::new());

            let plist = ProcessList {
                new: new.into_iter().map(|s| s.to_string()).collect(),
                del: del.into_iter().map(|s| s.to_string()).collect(),
            };

            let bcast = Broadcast(Dispatch::make_raw_announce(&plist, Uuid::nil()));
            let bcast = NodeControl::from_registry().send(bcast);
            ctx.spawn(wrap_future(async move { bcast.await.unwrap() }));
        });
    }
}

impl Handler<RecvFromNode<ProcessList>> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: RecvFromNode<ProcessList>, ctx: &mut Context<Self>) -> Self::Result {
        let node: Uuid = msg.node_id;
        log::info!("Received process update from remote node: {:?}", msg.node_id);

        for del in msg.inner.del {
            log::info!("Proc: {} no longer running", del);
            self.nodes.remove(&Uuid::from_str(&del).unwrap());
        }
        for new in msg.inner.new {
            log::info!("Proc: {} running on {}", new, node);
            self.nodes.insert(Uuid::from_str(&new).unwrap(), node.clone());
        }
    }
}

impl Handler<NodeUpdate> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: NodeUpdate, ctx: &mut Context<Self>) -> Self::Result {
        if let NodeUpdate::Connected(id) = msg {
            log::info!("Announcing process list to new node: {}", id);
            let control = NodeControl::from_registry();
            let update = ProcessList {
                new: self.local.keys().map(|k| k.to_string()).collect(),
                del: vec![],
            };
            let msg = SendToNode(id, Dispatch::make_raw_announce(&update, Uuid::nil()));
            let fut = control.send(msg);
            let fut = wrap_future(async move { fut.await.unwrap().unwrap(); });
            // TODO: Do we need to wait here ?
            ctx.wait(fut);
        }
    }
}

pub struct RegisterProcess {
    id: Uuid,
    dispatcher: Box<dyn Dispatcher>,
}

impl RegisterProcess {
    pub fn new<A: ProcessDispatch>(pid: Pid<A>) -> Self {
        let dispatcher = A::make_dispatcher(pid.local_addr().unwrap().downgrade());
        let id = pid.id();
        RegisterProcess {
            id,
            dispatcher,
        }
    }
}

impl Message for RegisterProcess {
    type Result = ();
}

impl Handler<RegisterProcess> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: RegisterProcess, ctx: &mut Context<Self>) -> Self::Result {
        self.new.insert(msg.id.clone());
        let _ = self.local.insert(msg.id, msg.dispatcher);
        // TODO: Send small eager updates when registering new processes, and don't wait for periodic update
    }
}

pub struct UnregisterProcess {
    pub id: Uuid
}

impl Message for UnregisterProcess {
    type Result = ();
}

impl Handler<UnregisterProcess> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: UnregisterProcess, ctx: &mut Context<Self>) -> Self::Result {
        log::info!("UNregistering {}", msg.id);
        self.local.remove(&msg.id);
        self.deleted.insert(msg.id);
    }
}

/// Dispatch a message to appropriate handler
///
/// if [id.is_nil() && !wait_for_response] then the response is returned as soon as local
/// link sent the message over the wire
///
/// Otherwise sets up a correlation counter and waits for response with a timeout(to prevent DOS attacks on correlation cache)
#[derive(Debug, Clone)]
pub struct Dispatch {
    pub id: Uuid,
    pub method: String,
    pub body: Bytes,
    pub wait_for_response: bool
}

impl Dispatch {
    /// Create a new dispatch from raw message, this message doesn't have to be an
    /// actual message, but rather any type that implements [prost::Message]
    pub fn make_raw_announce<M>(msg: &M, to: Uuid) -> Self
    where M: Wired
    {
        Dispatch {
            id: to,
            body: Wired::to_buf(msg).unwrap(),
            method: core::any::type_name::<M>().to_string(),
            wait_for_response: false,
        }
    }
    /// Create a new dispatch from raw message, this message doesn't have to be an
    /// actual message, but rather any type that implements [prost::Message]
    pub fn make_raw_request<M>(msg: &M, to: Uuid) -> Self
    where M: Wired
    {
        Dispatch {
            id: to,
            body: Wired::to_buf(msg).unwrap(),
            method: core::any::type_name::<M>().to_string(),
            wait_for_response: true,
        }
    }
}

impl Message for Dispatch {
    type Result = Result<Bytes, DispatchError>;
}

impl Handler<Dispatch> for ProcessRegistry {
    type Result = Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: Dispatch, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(p) = self.local.get(&msg.id) {
            Response::fut(p.dispatch(msg.method, msg.body))
        } else {
            if let Some(node) = self.nodes.get(&msg.id) {
                Response::fut(NodeControl::from_registry().send(SendToNode(node.clone(), msg)).map(|x| x.unwrap()))
            } else {
                Response::reply(Err(DispatchError::DispatchLocal))
            }
        }
    }
}
