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
use crate::node::NodeControl;
use bytes::Bytes;

#[doc(hidden)]
pub mod derive {
    pub use futures::future::BoxFuture;
    pub use crate::process::{ProcessDispatch, Dispatcher, DispatchError};
    pub use crate::util::Service;
    pub use bytes::{BytesMut, Bytes};
    pub use prost::Message as ProstMessage;
}

pub use _der::ProcessDispatch;
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
use crate::process::DispatchError;


#[derive(Debug, Clone)]
pub struct Broadcast {
    pub(crate) method: u64,
    pub(crate) body: Bytes,
}

impl Broadcast {
    pub fn make(method: u64, body: Bytes) -> Self {
        Self {
            method,
            body,
        }
    }
}

impl Message for Broadcast { type Result = (); }

/// Dispatch a message to appropriate handler
///
/// if `id.is_nil() && !wait_for_response` then the response is returned as soon as local
/// link sent the message over the wire
///
/// Otherwise sets up a correlation counter and waits for response with a timeout(to prevent DOS attacks on correlation cache)
#[derive(Debug, Clone)]
pub struct Dispatch {
    pub(crate) id: Uuid,
    pub(crate) method: u64,
    pub(crate) body: Bytes,
    pub(crate) wait_for_response: bool,
}

impl Message for Dispatch {
    type Result = Result<Bytes, DispatchError>;
}
