#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pid {
    #[prost(bytes, tag="1")]
    pub id: std::vec::Vec<u8>,
}
/// List of created/deleted process ids
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProcessList {
    #[prost(bytes, tag="2")]
    pub newids: std::vec::Vec<u8>,
    #[prost(bytes, tag="3")]
    pub delids: std::vec::Vec<u8>,
}

pub struct Update(pub ProcessList);
impl From<ProcessList> for Update {
    fn from(a: ProcessList) -> Self {
        Self(a)
    }
}
impl Into<ProcessList> for Update {
    fn into(self) -> ProcessList {
        self.0
    }
}
impl ::core::ops::Deref for Update {
    type Target = ProcessList;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for Update {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl crate::util::Service for Update {
    const NAME: &'static str = "quix.process.Process.Update";
     fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }
}
impl actix::Message for Update {
    type Result = ();
}
            
pub struct Get(pub ());
impl From<()> for Get {
    fn from(a: ()) -> Self {
        Self(a)
    }
}
impl Into<()> for Get {
    fn into(self) -> () {
        self.0
    }
}
impl ::core::ops::Deref for Get {
    type Target = ();
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for Get {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl crate::util::Service for Get {
    const NAME: &'static str = "quix.process.Process.Get";
     fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }
}
impl actix::Message for Get {
    type Result = Result<ProcessList, ()>;
}
            