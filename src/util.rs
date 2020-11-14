use actix::{Recipient, Message};
use uuid::Uuid;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::TryInto;
use crate::process::registry::Dispatch;

pub struct RegisterRecipient<M>(pub Recipient<M>)
where M: Message + Send,
      M::Result: Send
;

impl<M> Message for RegisterRecipient<M> where M: Message + Send,
                                               M::Result: Send
{ type Result = Result<Uuid, std::convert::Infallible>; }


pub trait Service: Sized {
    const NAME: &'static str;

    fn read(b: impl Buf) -> Result<Self, ()>;
    fn write(&self, b: &mut impl BufMut) -> Result<(), ()>;

    fn to_buf(&self) -> Result<Bytes, ()> {
        let mut b = BytesMut::new();
        self.write(&mut b)?;
        return Ok(b.freeze());
    }

    fn make_ann_dispatch(&self, to: Uuid) -> Result<Dispatch, ()> {
        Ok(Dispatch {
            id: to,
            body: Service::to_buf(self)?,
            method: Self::NAME.to_string(),
            wait_for_response: false,
        })
    }
    fn make_call_dispatch(&self, to: Uuid) -> Result<Dispatch, ()> {
        Ok(Dispatch {
            id: to,
            body: Service::to_buf(self)?,
            method: Self::NAME.to_string(),
            wait_for_response: true,
        })
    }
}

pub fn uuid(data: impl AsRef<[u8]>) -> Uuid {
    Uuid::from_bytes(data.as_ref().try_into().unwrap())
}