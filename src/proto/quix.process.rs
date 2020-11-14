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

use quix::derive::*;
pub struct Update(pub ProcessList);

impl actix::Message for Update {
    type Result = ();
}

impl quix::derive::Service for Update {
    const NAME: &'static str = "quix.process.Process.update";
    const ID: u64 = 5537769940034032186;
    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        prost::Message::encode(&self.0, b).map_err(|_| ())
    }
    fn read(b: impl bytes::Buf) -> Result<Self, ()> {
        Ok(Self(prost::Message::decode(b).map_err(|_| ())?))
    }

    fn read_result(b: impl bytes::Buf) -> Result<Self::Result, ()> {
        Ok(())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), ()> {
        ();
        Ok(())
    }
}

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
            