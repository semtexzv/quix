use crate::import::*;


use crate::process::{Dispatcher, DynHandler, Pid, Process, DispatchError};
use crate::node::{NodeController, RegisterGlobalHandler, FromNode, NodeStatus};
use crate::util::{RegisterRecipient, RpcMethod};
use crate::proto::{Update, ProcessList, InfoOf};
use crate::{Dispatch, NodeDispatch, MethodCall, ProcDispatch};

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

fn fold_uuids(mut a: Vec<u8>, u: &Uuid) -> Vec<u8> {
    a.extend_from_slice(u.as_bytes());
    a
}

impl Supervised for ProcessRegistry {
    fn restarting(&mut self, ctx: &mut Self::Context) {
        log::info!("Setting up process registry");
        let control = NodeController::from_registry();

        control.do_send(RegisterRecipient(ctx.address().recipient::<NodeStatus>()));
        control.do_send(RegisterGlobalHandler::new::<Update, _>(ctx.address().recipient()));
        control.do_send(RegisterGlobalHandler::new::<InfoOf, _>(ctx.address().recipient()));

        ctx.run_interval(Duration::from_millis(800), |this, ctx| {
            if this.new.is_empty() && this.deleted.is_empty() {
                return;
            }

            log::info!("Broadcasting process table update");
            let new = std::mem::replace(&mut this.new, HashSet::new());
            let del = std::mem::replace(&mut this.deleted, HashSet::new());

            let plist = ProcessList {
                newids: new.iter().fold(vec![], fold_uuids),
                delids: del.iter().fold(vec![], fold_uuids),
            };

            let bcast = Update(plist).make_broadcast();

            let bcast = NodeController::from_registry().do_send(bcast);
        });
    }
}

impl Handler<FromNode<InfoOf>> for ProcessRegistry {
    type Result = Result<crate::proto::Pid, DispatchError>;

    fn handle(&mut self, msg: FromNode<InfoOf>, ctx: &mut Context<Self>) -> Self::Result {
        Ok(crate::proto::Pid { id: Vec::new() })
    }
}

impl Handler<FromNode<Update>> for ProcessRegistry {
    type Result = Result<(), DispatchError>;

    fn handle(&mut self, msg: FromNode<Update>, ctx: &mut Context<Self>) -> Self::Result {
        let node: Uuid = msg.node_id;
        log::info!("Received process update from remote node: {:?}", msg.node_id);

        for del in msg.inner.delids.array_chunks::<16>() {
            let del = &Uuid::from_bytes(*del);
            log::info!("Proc: {} no longer running", del);
            self.nodes.remove(del);
        }
        for new in msg.inner.newids.array_chunks::<16>() {
            let new = Uuid::from_bytes(*new);
            log::info!("Proc: {} running on {}", new, node);
            self.nodes.insert(new, node);
        }
        Ok(())
    }
}

impl Handler<NodeStatus> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: NodeStatus, ctx: &mut Context<Self>) -> Self::Result {
        if let NodeStatus::Connected(id) = msg {
            log::info!("Announcing process list to new node: {}", id);
            let control = NodeController::from_registry();
            let update = ProcessList {
                newids: self.local.keys().fold(vec![], fold_uuids),
                delids: vec![],
            };

            let msg = NodeDispatch {
                nodeid: id,
                inner: Update(update).make_announcement(),
            };

            let fut = control.send(msg);
            let fut = wrap_future(async move { fut.await.unwrap().unwrap(); });
            // TODO: Do we need to wait here ?
            ctx.wait(fut);
        }
    }
}

pub struct Register {
    id: Uuid,
    dispatcher: Box<dyn Dispatcher>,
}

impl Register {
    pub fn new<A: DynHandler>(pid: Pid<A>) -> Self {
        let dispatcher = A::make_dispatcher(pid.local_addr().unwrap().downgrade());
        let id = pid.id();
        Register {
            id,
            dispatcher,
        }
    }
}

impl Message for Register { type Result = (); }

impl Handler<Register> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: Register, ctx: &mut Context<Self>) -> Self::Result {
        self.new.insert(msg.id.clone());
        let _ = self.local.insert(msg.id, msg.dispatcher);
        // TODO: Send small eager updates when registering new processes, and don't wait for periodic update
    }
}

pub struct Unregister {
    pub id: Uuid
}

impl Message for Unregister {
    type Result = ();
}

impl Handler<Unregister> for ProcessRegistry {
    type Result = ();

    fn handle(&mut self, msg: Unregister, ctx: &mut Context<Self>) -> Self::Result {
        log::info!("UNregistering {}", msg.id);
        self.local.remove(&msg.id);
        self.deleted.insert(msg.id);
    }
}

impl Handler<ProcDispatch<MethodCall>> for ProcessRegistry {
    type Result = Response<Bytes, DispatchError>;

    fn handle(&mut self, msg: ProcDispatch<MethodCall>, ctx: &mut Context<Self>) -> Self::Result {
        // TODO: Separate handling of dispatch coming from other nodes, prevent cycles
        if let Some(p) = self.local.get(&msg.procid) {
            Response::fut(p.dispatch(msg.inner.method, msg.inner.body))
        } else {
            if let Some(node) = self.nodes.get(&msg.procid) {
                let msg = NodeDispatch {
                    nodeid: *node,
                    inner: msg.inner,
                };
                Response::fut(NodeController::from_registry().send(msg).map(|x| x.unwrap()))
            } else {
                Response::reply(Err(DispatchError::DispatchLocal))
            }
        }
    }
}
