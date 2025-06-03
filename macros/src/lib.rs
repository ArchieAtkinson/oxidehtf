use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, Visibility, parse_macro_input, token};

#[proc_macro_attribute]
pub fn tests(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemMod);

    let mod_name = &input.ident;
    let mut test_functions_pointers = Vec::new();
    let mut test_functions_names = Vec::new();
    let mut processed_items = Vec::new();

    if let Some((brace, items)) = input.content.take() {
        for item in items {
            if let syn::Item::Fn(mut func) = item {
                let mut is_test_fn = false;

                // Filter out the #[test] attribute
                func.attrs.retain(|attr| {
                    if attr.path().is_ident("test") {
                        is_test_fn = true;
                        false // Remove this attribute
                    } else {
                        true // Keep other attributes
                    }
                });

                if is_test_fn {
                    // Make the function public as main is outside of test mod
                    func.vis = Visibility::Public(token::Pub::default());
                    let func_name = &func.sig.ident;
                    test_functions_pointers.push(quote! { #mod_name::#func_name});
                    test_functions_names.push(quote! {stringify!(#func_name)});
                }
                processed_items.push(syn::Item::Fn(func)); // Add the modified function
            } else {
                processed_items.push(item); // Add other items as is
            }
        }
        input.content = Some((brace, processed_items));
    }

    let expanded = quote! {
        #input // This now contains the module with #[test] attributes removed and functions made public

        fn main() -> color_eyre::eyre::Result<()> {

            use crate::#mod_name::Fixture;

            let (funcs, data) = htf2::gen_test_data(
                vec![#(#test_functions_pointers),*],
                vec![#(#test_functions_names),*]);
            let context = Fixture::default();
            htf2::run_tests(funcs, data, context)

        }
    };

    expanded.into()
}
