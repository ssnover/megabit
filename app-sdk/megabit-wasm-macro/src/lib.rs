use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

#[proc_macro_attribute]
pub fn megabit_wasm_app(_args: TokenStream, input: TokenStream) -> TokenStream {
    wasm_app_inner(input.into()).into()
}

fn wasm_app_inner(input: TokenStream2) -> TokenStream2 {
    let type_name = parse_type_name(input.clone())
        .expect("#[megabit_wasm_app] macro must decorate a struct, enum, or type.");

    quote! {
        #input

        mod __wasm_app_inner_proc_macro {
            use super::#type_name;
            use extism_pdk::{plugin_fn, FnResult};
            use megabit_app_sdk::MegabitApp;

            static mut APP_SINGLETON: Option<#type_name> = None;

            #[plugin_fn]
            pub fn setup() -> FnResult<()> {
                let display_cfg = megabit_app_sdk::display::get_display_info()?;
                let app = #type_name::setup(display_cfg)?;
                unsafe { APP_SINGLETON.replace(app); }
                Ok(())
            }

            #[plugin_fn]
            pub fn run() -> FnResult<()> {
                unsafe {
                    APP_SINGLETON.as_mut().unwrap().run()
                }
            }
        }
    }
}

fn parse_type_name(input_stream: TokenStream2) -> Option<Ident> {
    if let Ok(syn::ItemStruct { ident, .. }) = syn::parse2(input_stream.clone()) {
        return Some(ident);
    }
    if let Ok(syn::ItemEnum { ident, .. }) = syn::parse2(input_stream.clone()) {
        return Some(ident);
    }
    if let Ok(syn::ItemType { ident, .. }) = syn::parse2(input_stream) {
        return Some(ident);
    }

    return None;
}
