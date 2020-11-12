use actix::{Recipient, Message};
use uuid::Uuid;

pub struct RegisterRecipient<M>(pub Recipient<M>)
where M: Message + Send,
      M::Result: Send
;

impl<M> Message for RegisterRecipient<M> where M: Message + Send,
                                               M::Result: Send
{ type Result = Result<Uuid, std::convert::Infallible>; }