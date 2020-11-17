// needed to use normal codegen
pub use crate as quix;

include!("./quix.net.rs");
include!("./quix.process.rs");
include!("./quix.memkv.rs");

impl<A> Into<Pid<A>> for PidProto
where A: DynHandler
{
    fn into(self) -> Pid<A> {
        Pid::Remote(crate::util::uuid(self.pid))
    }
}

impl<A> From<Pid<A>> for PidProto
where A: DynHandler
{
    fn from(p: Pid<A>) -> Self {
        Self {
            pid: p.id().as_bytes().to_vec()
        }
    }
}


impl<M> Into<PidRecipient<M>> for PidProto
where M: Message + Send + 'static,
      M::Result: Send
{
    fn into(self) -> PidRecipient<M> {
        PidRecipient {
            id: crate::util::uuid(self.pid),
            local: None,
        }
    }
}

impl<M> From<PidRecipient<M>> for PidProto
where M: Message + Send + 'static,
      M::Result: Send
{
    fn from(p: PidRecipient<M>) -> Self {
        Self {
            pid: p.id.as_bytes().to_vec()
        }
    }
}