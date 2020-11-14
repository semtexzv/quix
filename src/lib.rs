#![feature(trait_alias)]
#![feature(core_intrinsics)]
#![feature(array_chunks)]
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
    pub use crate::util::Wired;
    pub use bytes::{BytesMut, Bytes};
}


pub mod proto;
pub mod process;
pub mod node;
pub mod util;
pub mod suspend;
pub mod global;

pub use _der::ProcessDispatch;

pub use process::{Pid, Process};

