use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemMod, LitInt, Visibility, parse_macro_input, token};

enum FuncKind {
    None,
    Test,
    FixtureInit,
}

#[proc_macro_attribute]
pub fn tests(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut value: usize = 0; // Default value if no input is present

    if !attr.is_empty() {
        // Attempt to parse the attribute input
        let parsed_attr = syn::parse::<LitInt>(attr.clone());

        match parsed_attr {
            Ok(lit_int) => {
                // Successfully parsed as LitInt, now try to convert to usize
                match lit_int.base10_parse::<usize>() {
                    Ok(parsed_value) => {
                        value = parsed_value;
                    }
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
            Err(_) => {
                panic!("Attribute must be a usize integer literal or empty.");
            }
        }
    }

    let mut input = parse_macro_input!(item as ItemMod);

    let mut fixture_init: Option<ItemFn> = None;
    let mod_name = &input.ident;
    let mut test_functions_pointers = Vec::new();
    let mut test_functions_names = Vec::new();
    let mut processed_items = Vec::new();

    if let Some((brace, items)) = input.content.take() {
        for item in items {
            if let syn::Item::Fn(mut func) = item {
                let mut func_kind = FuncKind::None;

                // Filter out the #[test] attribute
                func.attrs.retain(|attr| {
                    if attr.path().is_ident("test") {
                        func_kind = FuncKind::Test;
                        false
                    } else if attr.path().is_ident("fixture") {
                        func_kind = FuncKind::FixtureInit;
                        false
                    } else {
                        true
                    }
                });

                match func_kind {
                    FuncKind::Test => {
                        func.vis = Visibility::Public(token::Pub::default());
                        let func_name = &func.sig.ident;
                        test_functions_pointers.push(quote! { #func_name});
                        test_functions_names.push(quote! {stringify!(#func_name)});
                        processed_items.push(syn::Item::Fn(func));
                    }
                    FuncKind::FixtureInit => {
                        func.vis = Visibility::Public(token::Pub::default());
                        fixture_init = Some(func.clone());
                        // fixture_type = match &func.sig.output {
                        //     ReturnType::Default => None,
                        //     ReturnType::Type(.., ty) => Some(ty.clone()),
                        // };

                        processed_items.push(syn::Item::Fn(func));
                    }
                    _ => (),
                }
            } else {
                processed_items.push(item); // Add other items as is
            }
        }
        input.content = Some((brace, processed_items));
    }

    let fixture_init_ident = if let Some(fixture_init) = fixture_init {
        let sig = fixture_init.sig.ident;
        Some(quote!(#sig))
    } else {
        None
    };

    let register = quote! {
        fn create_suite_inventory() -> oxidehtf::TestSuiteBuilder {
            oxidehtf::TestSuiteBuilder::new(
                vec![#(#test_functions_pointers),*],
                crate::#mod_name::#fixture_init_ident,
                vec![#(#test_functions_names),*],
                stringify!(#mod_name),
                #value)
        }

        inventory::submit!{
            oxidehtf::TestSuiteBuilderProducer::new(create_suite_inventory)
        }

    };

    struct ItemsParser {
        items: Vec<syn::Item>,
    }

    impl syn::parse::Parse for ItemsParser {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            let mut items = Vec::new();
            while !input.is_empty() {
                items.push(input.parse()?);
            }
            Ok(ItemsParser { items })
        }
    }

    let parsed_items_wrapper: ItemsParser =
        syn::parse2(register).expect("Failed to parse TokenStream into multiple Items");

    input
        .content
        .as_mut()
        .expect("Content should be present")
        .1
        .extend(parsed_items_wrapper.items);

    quote! {#input}.into()
}
