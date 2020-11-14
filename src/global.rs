use actix::{Message, Handler, Actor, Supervised, SystemService, Context};
use std::convert::Infallible;
use serde::export::PhantomData;

#[derive(Default)]
pub struct Global<T> (pub T);

impl<T: Unpin + Send + 'static> Actor for Global<T> {
    type Context = Context<Self>;
}

impl<T: Unpin + Send + 'static> Supervised for Global<T> {}

impl<T: Default + Unpin + Send + 'static> SystemService for Global<T> {}


pub struct Get<T>(pub PhantomData<T>);

impl<T> Default for Get<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Clone + Unpin + Send + 'static> Message for Get<T> { type Result = Result<T, Infallible>; }

impl<T: Clone + Unpin + Send + 'static> Handler<Get<T>> for Global<T> {
    type Result = Result<T, Infallible>;

    fn handle(&mut self, msg: Get<T>, ctx: &mut Self::Context) -> Self::Result {
        Ok(self.0.clone())
    }
}

pub struct Set<T: Unpin + Send + 'static> (pub T);

impl<T: Unpin + Send + 'static> Message for Set<T> { type Result = (); }

impl<T: Unpin + Send + 'static> Handler<Set<T>> for Global<T> {
    type Result = ();

    fn handle(&mut self, msg: Set<T>, ctx: &mut Self::Context) -> Self::Result {
        self.0 = msg.0
    }
}