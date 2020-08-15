use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse::Nothing, parse_macro_input, Arm, ExprMatch, ItemFn, Pat, Signature};

#[proc_macro_error]
#[proc_macro_attribute]
pub fn cmd(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);

    let ItemFn {
        attrs,
        block,
        sig,
        vis,
    } = parse_macro_input!(input as ItemFn);

    let Signature {
        ident,
        inputs,
        output,
        ..
    } = sig;

    let call = match inputs.len() {
        0 => quote!(inner()),
        1 => quote!(inner(_ctx)),
        2 => quote!(inner(_ctx, _msg)),
        3 => quote!(inner(_ctx, _msg, _args)),
        _ => abort!(inputs, "Too many arguments in #[cmd] function."),
    };

    let expanded = quote! {
        #[serenity::framework::standard::macros::command]
        #(#attrs)*
        #vis fn #ident(
            _ctx: &mut serenity::client::Context,
            _msg: &serenity::model::channel::Message,
            mut _args: serenity::framework::standard::Args)
            -> serenity::framework::standard::CommandResult {
                let _ctx = &*_ctx;
                #[fehler::throws(anyhow::Error)]
                fn inner(#inputs) #output
                #block
                crate::error::wrap_cmd_err(|| #call)
            }
    };

    TokenStream::from(expanded)
}

#[proc_macro_error]
#[proc_macro]
pub fn shortcuts(input: TokenStream) -> TokenStream {
    let ExprMatch { arms, .. } = parse_macro_input!(input as ExprMatch);

    let mut shorts = vec![];
    let mut longs = vec![];
    for Arm {
        pat, body: long, ..
    } in arms
    {
        let short = if let Pat::Ident(pat_ident) = pat {
            pat_ident.ident
        } else {
            abort!(pat, "expecting ident")
        };
        shorts.push(short);

        longs.push(long);
    }

    let expanded = quote! {
        #[serenity::framework::standard::macros::group]
        #[commands(#(#shorts),*)]
        struct Shortcut;

        #(
            #[serenity::framework::standard::macros::command]
            #[help_available(false)]
            fn #shorts(
                _ctx: &mut serenity::client::Context,
                _msg: &serenity::model::channel::Message,
                _args: serenity::framework::standard::Args
            ) -> serenity::framework::standard::CommandResult
            {
                #longs(_ctx, _msg, _args)
            }
        )*
    };

    TokenStream::from(expanded)
}
