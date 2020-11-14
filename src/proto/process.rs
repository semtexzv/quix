// Automatically generated rust module for 'process.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use core::{convert::{TryFrom, TryInto}, ops::{Deref, DerefMut}};use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Pid<'a> {
    pub id: Cow<'a, [u8]>,
}

impl<'a> MessageInfo for Pid<'a> {
    const PATH : &'static str = "quix.process.Pid";
}

impl<'a> MessageRead<'a> for Pid<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Pid<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.id == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.id).len()) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.id != Cow::Borrowed(b"") { w.write_with_tag(10, |w| w.write_bytes(&**&self.id))?; }
        Ok(())
    }
}


            #[derive(Debug)]
            struct PidOwnedInner {
                buf: Vec<u8>,
                proto: Pid<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl PidOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Pid::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Pid<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct PidOwned {
                inner: core::pin::Pin<Box<PidOwnedInner>>,
            }

            impl MessageInfo for PidOwned {
                const PATH: &'static str = "quix.process.Pid";
            }

            #[allow(dead_code)]
            impl PidOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &Pid {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for PidOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for PidOwned {
                type Target = Pid<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for PidOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for PidOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: PidOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for PidOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<Pid<'a>> for PidOwned {
                fn from(proto: Pid) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: PidOwnedInner::new(buf).unwrap() }
                }
            }
            
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProcessList<'a> {
    pub newids: Vec<Cow<'a, str>>,
    pub delids: Vec<Cow<'a, str>>,
}

impl<'a> MessageInfo for ProcessList<'a> {
    const PATH : &'static str = "quix.process.ProcessList";
}

impl<'a> MessageRead<'a> for ProcessList<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(18) => msg.newids.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(26) => msg.delids.push(r.read_string(bytes).map(Cow::Borrowed)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for ProcessList<'a> {
    fn get_size(&self) -> usize {
        0
        + self.newids.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
        + self.delids.iter().map(|s| 1 + sizeof_len((s).len())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.newids { w.write_with_tag(18, |w| w.write_string(&**s))?; }
        for s in &self.delids { w.write_with_tag(26, |w| w.write_string(&**s))?; }
        Ok(())
    }
}


            #[derive(Debug)]
            struct ProcessListOwnedInner {
                buf: Vec<u8>,
                proto: ProcessList<'static>,
                _pin: core::marker::PhantomPinned,
            }

            impl ProcessListOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: unsafe { core::mem::MaybeUninit::zeroed().assume_init() },
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = ProcessList::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, ProcessList<'static>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = proto;
                    }
                    Ok(pinned)
                }
            }

            pub struct ProcessListOwned {
                inner: core::pin::Pin<Box<ProcessListOwnedInner>>,
            }

            impl MessageInfo for ProcessListOwned {
                const PATH: &'static str = "quix.process.ProcessList";
            }

            #[allow(dead_code)]
            impl ProcessListOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto(&self) -> &ProcessList {
                    &self.inner.proto
                }
            }

            impl core::fmt::Debug for ProcessListOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.fmt(f)
                }
            }

            impl Deref for ProcessListOwned {
                type Target = ProcessList<'static>;

                fn deref(&self) -> &Self::Target {
                    &self.inner.proto
                }
            }

            impl DerefMut for ProcessListOwned {
                fn deref_mut(&mut self) -> &mut Self::Target {
                    unsafe { &mut self.inner.as_mut().get_unchecked_mut().proto }
                }
            }

            impl TryFrom<Vec<u8>> for ProcessListOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: ProcessListOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for ProcessListOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.deref().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            #[cfg(feature = "test_helpers")]
            impl<'a> From<ProcessList<'a>> for ProcessListOwned {
                fn from(proto: ProcessList) -> Self {
                    use quick_protobuf::{MessageWrite, Writer};

                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    proto.write_message(&mut writer).expect("bad proto serialization");
                    Self { inner: ProcessListOwnedInner::new(buf).unwrap() }
                }
            }
            
