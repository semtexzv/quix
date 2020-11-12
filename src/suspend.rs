use actix::Actor;
use crate::Process;
use futures::Future;
use crate::process::ProcessDispatch;

/// Actor, which can be suspended to disk and woken up by loading it
pub trait Suspendable: Actor<Context=Process<Self>> + ProcessDispatch + prost::Message {
    /// Do work before actor is suspended. The returned future will be waited upon, and
    /// will block any further message processing
    fn on_suspend(&mut self, ctx: &mut Self::Context) -> Box<dyn Future<Output=()>>;

    /// Resume execution from suspended state
    fn on_resume(&mut self, ctx: &mut Self::Context);
}