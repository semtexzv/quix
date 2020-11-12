use actix::{Actor, Addr, Message, Handler, ActorFuture, AsyncContext, SpawnHandle, ActorContext, ActorState, Context, Supervised, SystemService, Recipient, WeakAddr};
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
use actix::prelude::Request;
use crate::process::registry::{Dispatch, ProcessRegistry, RegisterProcess, UnregisterProcess};
use crate::node::{NodeControl, SendToNode, encode};
use futures::{FutureExt, TryFutureExt};
use tokio::macros::support::Pin;
use futures::task::Poll;
use std::task;
use crate::derive::ProstMessage;

pub mod registry;

#[derive(Debug)]
pub enum DispatchError {
    TimeoutLocal,
    TimeoutRemote,
    DispatchRemote,
    DispatchLocal,
    MailboxRemote,
    Format,
}

/// Trait used to get generic dispatchers for different actors
pub trait Dispatcher: Send + 'static {
    /// Lookup the method, deserialize to proper type, execute, serialize and return
    fn dispatch(&self, method: String, data: Bytes) -> BoxFuture<'static, Result<Bytes, DispatchError>>;
}

/// Trait which must be implemented for all processes.
///
/// The implementation of this trait is responsible for serializing/deserializing messages into proper structures,
/// and should by implemnted by using the proc macro
pub trait ProcessDispatch: Actor<Context=Process<Self>> {
    fn make_dispatcher(addr: WeakAddr<Self>) -> Box<dyn Dispatcher>;
}

/// A special execution context. In this context the actor has a stable identity,
/// whuch should not change. It can also receive messages from remote nodes.
pub struct Process<A: Actor<Context=Self>> where A: ProcessDispatch
{
    id: Uuid,
    parts: ContextParts<A>,
    mb: Option<Mailbox<A>>,
}

impl<A: Actor<Context=Self>> Process<A> where A: ProcessDispatch
{
    /// Start a new process
    pub fn start(a: A) -> Pid<A> {
        Self::start_with(|_| a)
    }

    /// Start a new process, with the ability to manipiulate its context before  actual startup
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

    /// Get [Pid] of current process
    pub fn pid(&self) -> Pid<A> {
        return Pid::Local {
            id: self.id.clone(),
            addr: self.parts.address(),
        };
    }

    fn run(mut self, act: A) -> Pid<A> {
        let id = self.id;
        let pid = self.pid();
        let fut = self.into_fut(act);
        // Register this process with registry when starting
        ProcessRegistry::from_registry().do_send(RegisterProcess::new(pid.clone()));
        let fut = fut.map(move |_| {
            ProcessRegistry::from_registry().do_send(UnregisterProcess { id });
        });
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

/// Global process identifier. This can be used to send messages to actors on different nodes.
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

    fn send<M>(&self, m: M) -> RemoteRequest<M>
    where M: Message + prost::Message,
          M::Result: prost::Message,
          A: Handler<M>
    {
        let dispatch = Dispatch::make_raw_request(&m, self.id.clone());

        RemoteRequest {
            rec: ProcessRegistry::from_registry().send(dispatch),
            _p: PhantomData,
        }
    }
    // TODO: Add do_send
}

/// Request to send message to remote process
///
/// This can only be used to send addressed messages
pub struct RemoteRequest<M: Message> {
    rec: Request<ProcessRegistry, Dispatch>,
    _p: PhantomData<M>,
}


impl<M: Message> Future for RemoteRequest<M>
where M: ProstMessage + Unpin,
      M::Result: ProstMessage + Default
{
    type Output = Result<M::Result, DispatchError>;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        match futures::ready!(self.get_mut().rec.poll_unpin(cx)) {
            Ok(Ok(res)) => {
                Poll::Ready(<M::Result as prost::Message>::decode(res).map_err(|e| DispatchError::Format))
            }
            Ok(Err(err)) => Poll::Ready(Err(err)),
            Err(mailbox) => Poll::Ready(Err(DispatchError::DispatchRemote)),
        }
    }
}