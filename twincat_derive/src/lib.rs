use std::str::FromStr;

use quote::quote;

#[proc_macro_attribute]
pub fn path_verify(
    attribute: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let function = proc_macro2::TokenStream::from(item.clone());

    let function_ident = syn::parse_macro_input!(item as syn::ItemFn).sig.ident;
    let mod_name = quote::format_ident!("test_ads_path_verify_{function_ident}");
    let test_name = quote::format_ident!("ads_path_verify_{function_ident}");

    let (client, ranges) = get_client_and_ranges(attribute);
    let inner = construct_loop(function_ident, ranges);

    quote!(
        #function

        #[cfg(test)]
        mod #mod_name {
            use super::*;

            #[test]
            #[serial_test::serial]
            fn #test_name() {
                let client = #client;

                #inner
            }
        }
    )
    .into()
}

fn get_client_and_ranges(
    attribute: proc_macro::TokenStream,
) -> (proc_macro2::TokenStream, Vec<proc_macro2::TokenStream>) {
    let client_and_ranges = attribute
        .to_string()
        .split(';')
        .map(|a| a.trim())
        .map(|a| proc_macro2::TokenStream::from_str(a).unwrap())
        .collect::<Vec<proc_macro2::TokenStream>>();

    let (client, range) = client_and_ranges
        .split_first()
        .expect("Please provide the client definition");

    (client.to_owned(), range.to_vec())
}

fn construct_loop(
    function_ident: syn::Ident,
    ranges: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let var_names = ranges
        .iter()
        .enumerate()
        .map(|(i, _)| quote::format_ident!("var{i}"))
        .collect::<Vec<syn::Ident>>();

    let mut inner = quote!(
        assert!(#function_ident(&client #(, #var_names)*).is_ok());
    );

    for (i, range) in ranges.iter().enumerate().rev() {
        let var_name = quote::format_ident!("var{i}");
        inner = quote!(
            for #var_name in #range {
                #inner
            }
        );
    }

    inner
}
