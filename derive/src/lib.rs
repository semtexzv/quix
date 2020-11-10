extern crate proc_macro;

use proc_macro::{TokenStream};

use syn::{DeriveInput, Attribute, Type};
use quote::quote;
use syn::parse::{Parse, ParseBuffer};
use syn::Token;

#[derive(Debug)]
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

#[proc_macro_derive(ProcessDispatch, attributes(dispatch))]
pub fn my_derive(_input: TokenStream) -> TokenStream {
    let mut i: DeriveInput = syn::parse(_input).unwrap();
    let attr: Attribute = i.attrs.pop().unwrap();
    let paths = if attr.path.get_ident().map(|i| i.to_string()).as_deref() == Some("dispatch") {
        let list: TypeList = syn::parse2(attr.tokens).unwrap();
        list.paths
    } else {
        panic!("Missing paths")
    };
    /*
    if $method.as_str() == type_name::<$tys>() {
            let msg = <$tys>::decode($data).map_err(|_| DispatchError {});
            return Box::pin(async move {
                let res = $addr.send(msg?).await.map_err(|_| DispatchError {})?;
                let mut buf = BytesMut::new();
                prost::Message::encode(&res, &mut buf).map_err(|_| DispatchError {})?;
                Ok(buf.freeze())
            });
        }
     */

    let cases = paths.into_iter().map(|p| {
        quote! {
            if method.as_str() == core::any::type_name::<#p>() {

                let msg = <#p as quix::derive::ProstMessage>::decode(data).map_err(|_| quix::derive::DispatchError {});
                let run = async move {
                    let res = addr.send(msg?).await.map_err(|_| quix::derive::DispatchError {})?;
                    let mut buf = quix::derive::BytesMut::new();
                    quix::derive::ProstMessage::encode(&res, &mut buf).map_err(|_| quix::derive::DispatchError {})?;
                    Ok(buf.freeze())
                };
                return Box::pin(run);
            }
        }
    });

    let name = i.ident;
    let tokens = quote! {
        impl quix::derive::ProcessDispatch for #name {
            fn make_dispatcher(addr: actix::Addr<Self>) -> Box<dyn quix::derive::Dispatcher> {
                pub struct LocalDispatcher { addr: actix::Addr<#name> }
                impl quix::derive::Dispatcher for LocalDispatcher {
                    fn dispatch(&self, method: String, data: quix::derive::Bytes) -> quix::derive::BoxFuture<'static, Result<quix::derive::Bytes, quix::derive::DispatchError>> {
                        use quix::derive::ProstMessage;
                        let addr = self.addr.clone();
                        #(#cases)*
                        return Box::pin(async move { Err(quix::derive::DispatchError {})})
                    }
                }
                return Box::new(LocalDispatcher { addr });
            }
        }
    };

    tokens.into()
}