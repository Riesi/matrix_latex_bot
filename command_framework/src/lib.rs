use proc_macro::{TokenStream};
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Item, Signature};

mod utils;
use utils::*;

#[proc_macro_attribute]
pub fn group(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_item: Item =  syn::parse(input).unwrap();
    let str = match parsed_item {
        Item::Struct(str) => str,
        _ =>  { panic!("Attributed token not a struct!") }
    };

    let mut description: String = String::new();
    let mut com_vec: Vec<Ident> = Vec::new();
    let mut com_vec_str: Vec<String> = Vec::new();

    for a in str.attrs{
        let val = parse_values(&a).unwrap();
        match &val.name.to_string()[..] {
            "commands" => {

                for com_fun in &val.literals{
                    if let syn::Lit::Str(fu) = com_fun {
                        com_vec.push(syn::Ident::new(&format!("{}_COMMAND_STRUCT", fu.value().to_string()), Span::call_site()));
                        com_vec_str.push(fu.value().to_string());
                    }
                }
            },
            "description" => if let syn::Lit::Str(desc) = val.literals.first().unwrap(){
                                description = desc.value();
                            }else { description = "".to_string() },
            _ => panic!("Unknown group attribute: {}!", val.name.to_string())

        }
    }

    let com_vec_size = com_vec.len();

    let name = str.ident;
    let visibility = str.vis;

    let name_ext = syn::Ident::new(&format!("{}_GROUP", name), Span::call_site());

    let expanded = quote! {
        lazy_static! {
            #visibility static ref #name_ext: ::command_structures::structures::GroupStruct = ::command_structures::structures::GroupStruct{
                commands: command_structures::structures::Handler {
                    command_list: {
                        let mut handler: ::std::collections::HashMap<String, &'static ::command_structures::structures::CommandStruct> = ::std::collections::HashMap::new();
                        #(
                            handler.insert(#com_vec_str.to_string() , &#com_vec);
                        )*
                        handler
                    },
                    max_command_size: #com_vec_size,
                },
                description: #description.to_string(),
            };
        }
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn command(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_item: Item =  syn::parse(input).unwrap();
    let fun = match parsed_item {
        Item::Fn(fun) => fun,
        _ =>  { panic!("Attributed token not a function!") }
    };

    let mut checks: Vec<Ident> = Vec::new();
    let mut description: String = String::new();

    for a in fun.attrs{
        let val = parse_values(&a).unwrap();
        match &val.name.to_string()[..] {
            "checks" => {
                for check_fun in &val.literals{
                    if let syn::Lit::Str(fu) = check_fun{
                        checks.push(syn::Ident::new(&format!("{}_CHECK", fu.value()), Span::call_site()));
                    }
                }
            },
            "description" => if let syn::Lit::Str(desc) = val.literals.first().unwrap(){
                                description = desc.value();},
            _ => panic!("Unknown command attribute: {}!", val.name.to_string())

        }
    }

    let Signature{ident: name, inputs: args, output: _ret, ..} = fun.sig;
    let body = fun.block.stmts;
    let visibility = fun.vis;

    let name_string = name.to_string();
    let name_ext = syn::Ident::new(&format!("{}_COMMAND", name), Span::call_site());

    let name_str = syn::Ident::new(&format!("{}_COMMAND_STRUCT", name), Span::call_site());

    let expanded = quote! {
        lazy_static! {
            pub static ref #name_str: ::command_structures::structures::CommandStruct = ::command_structures::structures::CommandStruct{
                description: #description.to_string(),
                name: #name_string.to_string(),
                fun: #name_ext as ::command_structures::structures::Command,
            };
        }
        #visibility fn #name_ext (#args) -> ::command_structures::structures::CommandResult{
            tokio::spawn(async move {//TODO get rid and implement proper returns
                let power_level: i64 = if let Some(member) =
                        room.get_member(&event.sender).await.expect("Failed to get member from UserId!") {
                    member.normalized_power_level()
                } else {
                    i64::MIN
                };
                #(
                if let Err(e) = #checks(power_level) {
                    return Err(::command_structures::structures::CommandFailed::CHECK);
                }
                )*
                #(#body)*
            });
            Ok(())
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn check(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let parsed_item: Item =  syn::parse(input).unwrap();
    let fun = match parsed_item {
        Item::Fn(fun) => fun,
        _ =>  { panic!("Attributed token not a function!") }
    };

    let Signature{ident: name, inputs: args, ..} = fun.sig;
    let name_ext = syn::Ident::new(&format!("{}_CHECK", name), Span::call_site());
    let body = fun.block.stmts;
    let visibility = fun.vis;

    let expanded = quote! {
        #visibility fn #name_ext (#args) -> ::command_structures::structures::CheckResult {
            #(#body)*
        }
    };

    expanded.into()
}
