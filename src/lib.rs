#![feature(trait_alias)]
#![feature(core_intrinsics)]
#![allow(unused)]
#![deny(unused_must_use)]

extern crate derive as _der;

use std::sync::Arc;
use actix::{Message, SystemService, Actor};
use crate::node::NodeControl;
use bytes::Bytes;

#[doc(hidden)]
pub mod derive {
    pub use futures::future::BoxFuture;
    pub use crate::process::{ProcessDispatch, Dispatcher, DispatchError};
    pub use prost::Message as ProstMessage;
    pub use bytes::{BytesMut, Bytes};
}


pub mod process;
pub mod node;
pub mod util;

pub use _der::ProcessDispatch;

pub use process::{Pid, Process};

