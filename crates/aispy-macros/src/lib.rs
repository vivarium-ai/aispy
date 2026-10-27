use proc_macro::TokenStream;

use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    Error,
    Expr,
    ExprLit,
    ItemFn,
    Lit,
    MetaNameValue,
    Result,
    Token,
};

struct ObserveArgs {
    kind: String,
}

impl Parse for ObserveArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let args =
            Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)?;

        let mut kind = None;

        for arg in args {
            if arg.path.is_ident("kind") {
                let Expr::Lit(ExprLit {
                    lit: Lit::Str(value),
                    ..
                }) = arg.value
                else {
                    return Err(Error::new_spanned(
                        arg.value,
                        "`kind` must be a string literal",
                    ));
                };

                kind = Some(value.value());
            } else {
                return Err(Error::new_spanned(
                    arg.path,
                    "unsupported observe argument",
                ));
            }
        }

        let kind = kind.ok_or_else(|| {
            Error::new(
                proc_macro2::Span::call_site(),
                "missing required `kind = \"...\"`",
            )
        })?;

        Ok(Self { kind })
    }
}

#[proc_macro_attribute]
pub fn observe(
    args: TokenStream,
    input: TokenStream,
) -> TokenStream {
    let args = parse_macro_input!(args as ObserveArgs);
    let mut function = parse_macro_input!(input as ItemFn);

    let aispy = match crate_name("aispy") {
        Ok(FoundCrate::Itself) => quote!(::aispy),

        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            quote!(::#ident)
        }

        Err(error) => {
            return Error::new(
                proc_macro2::Span::call_site(),
                format!("unable to locate the `aispy` crate: {error}"),
            )
            .into_compile_error()
            .into();
        }
    };

    let kind = args.kind;
    let function_name = function.sig.ident.to_string();
    let body = function.block;

    function.block = Box::new(
        syn::parse_quote!({
            let _aispy_observation =
                #aispy::observation::enter(
                    #aispy::observation::Site {
                        kind: #kind,
                        function: #function_name,
                        module: module_path!(),
                        file: file!(),
                        line: line!(),
                        column: column!(),
                    },
                );

            #body
        }),
    );

    quote!(#function).into()
}
