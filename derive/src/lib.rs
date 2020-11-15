extern crate proc_macro;

use proc_macro::{TokenStream};

use syn::{DeriveInput, Attribute, Type};
use quote::quote;
use syn::parse::{Parse, ParseBuffer};
use syn::Token;

struct TypeList {
    paths: Vec<Type>
}

impl Parse for TypeList {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let content;
        let mut res = vec![];
        syn::parenthesized!(content in input);
        loop {
            res.push(content.parse()?);
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            } else {
                break;
            }
        }
        Ok(Self {
            paths: res
        })
    }
}

#[proc_macro_derive(DynHandler, attributes(dispatch))]
pub fn my_derive(_input: TokenStream) -> TokenStream {
    let mut i: DeriveInput = syn::parse(_input).unwrap();
    let attr: Attribute = i.attrs.pop().unwrap();
    let paths = if attr.path.get_ident().map(|i| i.to_string()).as_deref() == Some("dispatch") {
        let list: TypeList = syn::parse2(attr.tokens).unwrap();
        list.paths
    } else {
        panic!("Missing paths")
    };
    let messages = paths.into_iter().map(|p| {
        let id = quote! { #p::ID };
        let block = quote! {
        {
            let msg = <#p as RpcMethod>::read(data).map_err(|_| quix::derive::DispatchError::Format);
            Box::pin(async move {
                let res = addr.send(msg?).await.map_err(|_| quix::derive::DispatchError::MailboxRemote)?;
                let mut buf = quix::derive::BytesMut::new();
                Ok(buf.freeze())
            })
        }
        };

        quote! { #id => #block }
    });


    let dispatch = quote! {
        match method {
            #(#messages),*
            _ => {
                Box::pin(async move { Err(quix::derive::DispatchError::DispatchRemote)})
            }
        }
    };

    let name = i.ident;

    let dispatcher = quote! {
        pub struct LocalDispatcher { addr: actix::WeakAddr<#name> }
        impl quix::derive::Dispatcher for LocalDispatcher {
            fn dispatch(&self, method: u32, data: quix::derive::Bytes) -> quix::derive::BoxFuture<'static, Result<quix::derive::Bytes, quix::derive::DispatchError>> {
                use quix::derive::RpcMethod;
                let addr = self.addr.upgrade().unwrap().clone();
                #dispatch
            }
        }
        Box::new(LocalDispatcher { addr })
    };

    let tokens = quote! {
        impl quix::derive::DynHandler for #name {
            fn make_dispatcher(addr: actix::WeakAddr<Self>) -> Box<dyn quix::derive::Dispatcher> {
                #dispatcher
            }
        }
    };

    tokens.into()
}