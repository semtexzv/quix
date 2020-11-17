use crate::import::*;
use crate::{Broadcast, MethodCall};
use crate::process::DispatchError;

pub struct RegisterRecipient<M>(pub Recipient<M>)
where M: Message + Send,
      M::Result: Send;

impl<M> Message for RegisterRecipient<M> where M: Message + Send,
                                               M::Result: Send
{ type Result = Result<Uuid, std::convert::Infallible>; }

pub trait RpcMethod: Sized + Message {
    const NAME: &'static str;
    // Unique ID of the service method. Should be crc64 of Name
    const ID: u32;

    fn read(b: impl Buf) -> Result<Self, DispatchError>;
    fn write(&self, b: &mut impl BufMut) -> Result<(), DispatchError>;

    fn read_result(b: impl Buf) -> Self::Result;
    fn write_result(r: &Self::Result, b: &mut impl BufMut) -> Result<(), DispatchError>;

    fn to_buf(&self) -> Result<Bytes, DispatchError> {
        let mut b = BytesMut::new();
        self.write(&mut b)?;
        return Ok(b.freeze());
    }

    fn make_broadcast(&self, id: Option<Uuid>) -> Broadcast {
        Broadcast {
            procid: id,
            method: Self::ID,
            body: RpcMethod::to_buf(self).unwrap(),
        }
    }

    fn make_call(&self, id: Option<Uuid>) -> MethodCall {
        MethodCall {
            procid: id,
            body: RpcMethod::to_buf(self).unwrap(),
            method: Self::ID,
        }
    }
}


pub fn uuid(data: impl AsRef<[u8]>) -> Uuid {
    Uuid::from_bytes(data.as_ref().try_into().unwrap())
}