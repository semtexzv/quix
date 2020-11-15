use crate::import::*;

use crate::Process;
use crate::process::DynHandler;
use crate::util::RpcMethod;

/// Actor, which can be suspended to disk and woken up by loading it
pub trait Suspendable: Actor<Context=Process<Self>> + DynHandler {
    type SuspendState : RpcMethod;

    /// Do work before actor is suspended. The returned future will be waited upon, and
    /// will block any further message processing
    fn suspend(&mut self, ctx: &mut Self::Context) -> Box<dyn Future<Output=Self::SuspendState>>;

    /// Recreate actor from suspended state
    fn reanimate(state : Self::SuspendState) -> Result<Self, ()>;
}