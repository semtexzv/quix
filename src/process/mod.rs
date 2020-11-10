use actix::{Actor, Addr, Message, Handler, ActorFuture, AsyncContext, SpawnHandle, ActorContext, ActorState, Context, Supervised, SystemService, Recipient};
use uuid::Uuid;
use actix::fut::wrap_future;
use std::marker::PhantomData;
use actix::dev::{Mailbox, ContextParts, ContextFut, AsyncContextParts, ToEnvelope, Envelope};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use futures::channel::oneshot::Sender;
use bytes::Bytes;
use futures::future::BoxFuture;

pub mod registry;

#[derive(Debug)]
pub struct DispatchError {}

pub trait Dispatcher: Send + 'static {
    fn dispatch(&self, method: String, data: Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>>;
}

/// Trait which must be implemented for all processes.
///
/// The implementation of this trait is responsible for serializing/deserializing messages, into proper structures,
/// and should by implemnted by using our proc macro
pub trait ProcessDispatch: Actor<Context=Process<Self>> {
    fn make_dispatcher(addr: Addr<Self>) -> Box<dyn Dispatcher>;
}

pub struct Process<A: Actor<Context=Self>> where A: ProcessDispatch
{
    id: Uuid,
    parts: ContextParts<A>,
    mb: Option<Mailbox<A>>,
}

impl<A: Actor<Context=Self>> Process<A> where A: ProcessDispatch
{
    pub fn start(a: A) -> Pid<A> {
        Self::start_with(|_| a)
    }

    pub fn start_with(f: impl FnOnce(&mut Self) -> A) -> Pid<A> {
        let (tx, rx) = actix::dev::channel::channel(2);
        // Global process registry
        let id = Uuid::new_v4();
        let parts = ContextParts::new(rx.sender_producer());
        let mut proc = Process {
            id,
            parts,
            mb: Some(actix::dev::Mailbox::new(rx)),
        };
        let act = f(&mut proc);
        proc.run(act)
    }

    pub fn pid(&self) -> Pid<A> {
        return Pid::Local {
            id: self.id.clone(),
            addr: self.parts.address(),
        };
    }

    fn run(self, act: A) -> Pid<A> {
        let pid = self.pid();
        let fut = self.into_fut(act);
        actix_rt::spawn(fut);
        pid
    }

    fn into_fut(mut self, act: A) -> ContextFut<A, Self> {
        let mb = self.mb.take().unwrap();
        ContextFut::new(self, act, mb)
    }
}

impl<A: Actor<Context=Self>> AsyncContextParts<A> for Process<A>
where A: ProcessDispatch
{
    fn parts(&mut self) -> &mut ContextParts<A> {
        &mut self.parts
    }
}

impl<A: Actor<Context=Self>> ActorContext for Process<A>
where A: ProcessDispatch
{
    fn stop(&mut self) {
        self.parts.stop()
    }

    fn terminate(&mut self) {
        self.parts.terminate()
    }

    fn state(&self) -> ActorState {
        self.parts.state()
    }
}

impl<A: Actor<Context=Self>> AsyncContext<A> for Process<A>
where A: ProcessDispatch
{
    fn address(&self) -> Addr<A> {
        self.parts.address()
    }

    fn spawn<F>(&mut self, fut: F) -> SpawnHandle where
        F: ActorFuture<Output=(), Actor=A> + 'static {
        self.parts.spawn(fut)
    }

    fn wait<F>(&mut self, fut: F) where
        F: ActorFuture<Output=(), Actor=A> + 'static {
        self.parts.wait(fut)
    }

    fn waiting(&self) -> bool {
        self.parts.waiting()
    }

    fn cancel_future(&mut self, handle: SpawnHandle) -> bool {
        self.parts.cancel_future(handle)
    }
}

impl<A: Actor<Context=Self>, M: Message> ToEnvelope<A, M> for Process<A>
where A: ProcessDispatch + Handler<M>,
      M: Send + 'static,
      M::Result: Send
{
    fn pack(msg: M, tx: Option<Sender<<M as Message>::Result>>) -> Envelope<A> {
        Envelope::new(msg, tx)
    }
}


pub enum Pid<A: Actor> {
    Local {
        id: Uuid,
        addr: Addr<A>,
    },
    Remote(Remote<A>),
}

impl<A: Actor> Clone for Pid<A> {
    fn clone(&self) -> Self {
        match self {
            Pid::Local {
                id, addr
            } => Pid::Local { id: id.clone(), addr: addr.clone() },
            Pid::Remote(r) => Pid::Remote(Clone::clone(&r)),
        }
    }
}


impl<A: Actor> Pid<A> {
    pub fn local_addr(&self) -> Option<Addr<A>> {
        match self {
            Pid::Local { addr, .. } => Some(addr.clone()),
            _ => None
        }
    }
    pub fn id(&self) -> Uuid {
        match self {
            Pid::Local { id, .. } => id.clone(),
            Pid::Remote(r) => r.id.clone()
        }
    }
}

pub struct Remote<A: Actor> {
    id: Uuid,
    _p: PhantomData<A>,
}

impl<A: Actor> Clone for Remote<A> {
    fn clone(&self) -> Self {
        Remote {
            id: self.id.clone(),
            _p: PhantomData,
        }
    }
}

impl<A: Actor> Remote<A> {
    fn id(&self) -> &Uuid {
        &self.id
    }
    fn send<M>(&self, m: M) -> impl ActorFuture<Actor=A, Output=M::Result>
    where M: Message,
          A: Handler<M>
    {
        wrap_future(async move { unimplemented!() })
    }
}
