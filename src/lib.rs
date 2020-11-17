#![feature(type_alias_impl_trait)]
#![feature(trait_alias)]
#![feature(core_intrinsics)]
#![feature(array_chunks)]
#![allow(unused)]
#![deny(unused_must_use)]

#[macro_use]
extern crate serde;

extern crate derive as _der;

use std::sync::Arc;
use actix::{Message, SystemService, Actor};
use crate::node::NodeController;
use bytes::Bytes;

#[doc(hidden)]
pub mod derive {
    pub use futures::FutureExt;
    pub use actix::prelude::*;
    pub use futures::future::BoxFuture;
    pub use bytes::{BytesMut, Bytes};
    pub use prost::Message as ProstMessage;

    pub use crate::process::{Pid, PidRecipient, DynHandler, Dispatcher};
    pub use crate::node::NodeId;
    pub use crate::util::RpcMethod;
    pub use crate::process::DispatchError;
}

pub use _der::DynHandler;
pub use process::{Pid, Process};

mod import;
pub mod proto;
pub mod process;
pub mod node;
pub mod util;
pub mod suspend;
pub mod global;
pub mod memkv;


use uuid::Uuid;
pub use crate::process::DispatchError;


#[derive(Debug, Clone)]
pub struct Broadcast {
    pub(crate) procid: Option<Uuid>,
    pub(crate) method: u32,
    pub(crate) body: Bytes,
}

impl Broadcast {
    pub fn make(procid: Option<Uuid>, method: u32, body: Bytes) -> Self {
        Self {
            procid,
            method,
            body,
        }
    }
}

impl Message for Broadcast {
    type Result = Result<(), DispatchError>;
}

// Send a specified message to a node
pub struct NodeDispatch<M> {
    pub(crate) nodeid: Uuid,
    pub(crate) inner: M,
}

impl<M, R> Message for NodeDispatch<M>
where M: Message<Result=Result<R, DispatchError>> + 'static,
      R: 'static
{
    type Result = Result<R, DispatchError>;
}
/*
// Send a specified message to a specified process
pub struct ProcDispatch<M> {
    pub(crate) procid: Uuid,
    pub(crate) inner: M,
}

impl<M, R> Message for ProcDispatch<M>
where M: Message<Result=Result<R, DispatchError>> + 'static,
      R: 'static
{
    type Result = Result<R, DispatchError>;
}
*/


//pub struct DynHandler<M> {}

/// Dispatch a message to appropriate handler
///
/// if `id.is_nil() && !wait_for_response` then the response is returned as soon as local
/// link sent the message over the wire
///
/// Otherwise sets up a correlation counter and waits for response with a timeout(to prevent DOS attacks on correlation cache)
#[derive(Debug, Clone)]
pub struct MethodCall {
    pub(crate) procid: Option<Uuid>,
    pub(crate) method: u32,
    pub(crate) body: Bytes,
}

impl Message for MethodCall {
    type Result = Result<Bytes, DispatchError>;
}