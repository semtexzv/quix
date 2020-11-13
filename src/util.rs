use actix::{Recipient, Message};
use uuid::Uuid;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct RegisterRecipient<M>(pub Recipient<M>)
where M: Message + Send,
      M::Result: Send
;

impl<M> Message for RegisterRecipient<M> where M: Message + Send,
                                               M::Result: Send
{ type Result = Result<Uuid, std::convert::Infallible>; }


pub trait Wired: Sized {
    fn to_buf(&self) -> Result<Bytes, ()> {
        let mut b = BytesMut::new();
        self.write(&mut b)?;
        return Ok(b.freeze());
    }
    fn write(&self, b: &mut impl BufMut) -> Result<(), ()>;
    fn read(b: impl Buf) -> Result<Self, ()>;
}

impl<T> Wired for T where T: prost::Message + Default {
    fn write(&self, b: &mut impl BufMut) -> Result<(), ()> {
        prost::Message::encode(self, b).map_err(|_| ())
    }

    fn read(b: impl Buf) -> Result<Self, ()> {
        prost::Message::decode( b).map_err(|_| ())
    }
}