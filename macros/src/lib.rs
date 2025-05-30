use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemMod, Visibility, parse_macro_input, token};

#[proc_macro_attribute]
pub fn tests(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemMod);

    let mod_name = &input.ident;
    let mut test_functions_pointers = Vec::new();
    let mut processed_items = Vec::new();

    // let mut foo = Vec::new();

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
                    // Generate the htf::Test::new call for this function
                    test_functions_pointers.push(quote! {
                        htf2::Test::new(#mod_name::#func_name, stringify!(#func_name))
                    });
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
            let tests = vec![
                #(#test_functions_pointers),*
            ];
            htf2::run_tests(tests)
        }
    };

    expanded.into()
}
