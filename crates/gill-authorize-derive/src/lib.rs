use proc_macro::TokenStream;
use quote::quote;
use syn::{parse, parse_quote, ItemFn};

#[proc_macro_attribute]
pub fn authorized(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse::<ItemFn>(item).expect("Expected a function");
    let mut statements = input_fn.block.stmts.clone();

    statements.insert(
        0,
        parse_quote! {
            let Some(user) = get_connected_user(&db, user).await else {
                return Err(AppError::Unauthorized);
            };
        },
    );

    input_fn.block.stmts = statements;
    TokenStream::from(quote! {#input_fn})
}
