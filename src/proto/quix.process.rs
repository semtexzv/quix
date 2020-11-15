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
    type Result = Result<(), DispatchError>;
}

impl quix::derive::RpcMethod for Update {

    const NAME: &'static str = "quix.process.Process.update";
    const ID: u32 = 520454116;

    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::MessageFormat)
    }
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::MessageFormat)?))
    }

    fn read_result(b: impl bytes::Buf) -> Self::Result {
        Ok(())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
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
            
use quix::derive::*;
pub struct InfoOf(pub Pid);

impl actix::Message for InfoOf {
    type Result = Result<Pid, DispatchError>;
}

impl quix::derive::RpcMethod for InfoOf {

    const NAME: &'static str = "quix.process.Process.info_of";
    const ID: u32 = 484571255;

    fn write(&self, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        prost::Message::encode(&self.0, b).map_err(|_| DispatchError::MessageFormat)
    }
    fn read(b: impl bytes::Buf) -> Result<Self, DispatchError> {
        Ok(Self(prost::Message::decode(b).map_err(|_| DispatchError::MessageFormat)?))
    }

    fn read_result(b: impl bytes::Buf) -> Self::Result {
        Ok(<Pid>::decode(b).unwrap())
    }

    fn write_result(res: &Self::Result, b: &mut impl bytes::BufMut) -> Result<(), DispatchError> {
        let a: &Pid = res.as_ref().unwrap(); a.encode(b).unwrap();
        Ok(())
    }
}

impl From<Pid> for InfoOf {
    fn from(a: Pid) -> Self {
        Self(a)
    }
}

impl Into<Pid> for InfoOf {
    fn into(self) -> Pid {
        self.0
    }
}

impl ::core::ops::Deref for InfoOf {
    type Target = Pid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ::core::ops::DerefMut for InfoOf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
            