use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, ItemFn, ItemMod, LitInt, PatType, ReturnType, Type, parse};

enum FuncKind {
    None,
    Test,
    FixtureInit,
}

#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    match tests_impl(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn check_fn_sig(sig: &syn::Signature) -> Result<(), ()> {
    if sig.constness.is_none()
        && sig.asyncness.is_none()
        && sig.unsafety.is_none()
        && sig.abi.is_none()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.variadic.is_none()
    {
        Ok(())
    } else {
        Err(())
    }
}

fn check_fixture_type(arg_type: Box<Type>, fixture_type: Box<Type>) -> bool {
    match *arg_type {
        Type::Reference(ry) if ry.mutability.is_some() => match (*ry.elem, *fixture_type) {
            (Type::Path(arg), Type::Path(fixture)) if arg == fixture => true,
            _ => false,
        },
        _ => false,
    }
}

// macro_rules! fn_state_signature_msg {
//     ($name:literal) => {
//         concat!(
//             "`#[",
//             $name,
//             "]` function must have signature `fn()` or `fn(state: &mut T)`"
//         )
//     };
// }

fn tests_impl(args: TokenStream, input: TokenStream) -> parse::Result<TokenStream> {
    let mut value: usize = 0; // Default value if no input is present

    if !args.is_empty() {
        // Attempt to parse the attribute input
        let parsed_args = syn::parse::<LitInt>(args.clone());

        match parsed_args {
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

    // let test_return_type: Type = parse_quote! { Result<(), oxidehtf::TestFailure> };

    let mut module: ItemMod = syn::parse(input)?;
    let mut fixture_init: Option<ItemFn> = None;
    let mod_name = &module.ident;
    let mut test_functions = Vec::new();
    let mut processed_items = Vec::new();

    if let Some((brace, items)) = module.content.take() {
        for item in items {
            let syn::Item::Fn(mut func) = item else {
                processed_items.push(item);
                continue;
            };

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
                    let has_return_type = match &func.sig.output {
                        ReturnType::Type(_, _) => true,
                        _ => false,
                    };

                    if check_fn_sig(&func.sig).is_err() || !has_return_type {
                        return Err(parse::Error::new(
                            func.sig.ident.span(),
                            "`#[test]` function must have signature `fn(&mut SysContext, &mut T) -> Result<(), TestFailure>`",
                        ));
                    }
                    test_functions.push(func.clone());
                    processed_items.push(syn::Item::Fn(func));
                }

                FuncKind::FixtureInit => {
                    if fixture_init.is_some() {
                        return Err(parse::Error::new(
                            func.sig.ident.span(),
                            "only a single `#[fixture]` function can be defined",
                        ));
                    }

                    let has_return_type = match &func.sig.output {
                        ReturnType::Type(_, _) => true,
                        _ => false,
                    };

                    if check_fn_sig(&func.sig).is_err()
                        || !func.sig.inputs.is_empty()
                        || !has_return_type
                    {
                        return Err(parse::Error::new(
                            func.sig.ident.span(),
                            "`#[fixture]` function must have signature `fn() -> T`",
                        ));
                    }

                    fixture_init = Some(func.clone());

                    processed_items.push(syn::Item::Fn(func));
                }
                _ => (),
            }
        }
        module.content = Some((brace, processed_items));
    }

    let fixture_init_ident = if let Some(fixture_init) = fixture_init.clone() {
        let sig = fixture_init.sig.ident;
        Some(quote!(#sig))
    } else {
        return Err(parse::Error::new(
            mod_name.span(),
            "Suites require an fixture (but can be empty)",
        ));
    };

    let test_functions_pointers: Vec<proc_macro2::TokenStream> = test_functions
        .iter()
        .map(|f| {
            let func_name = f.sig.ident.clone();
            quote! {#func_name}
        })
        .collect();

    let test_functions_names: Vec<proc_macro2::TokenStream> = test_functions
        .iter()
        .map(|f| {
            let func_name = f.sig.ident.clone();
            quote! {stringify!(#func_name)}.into()
        })
        .collect();

    let fixture_type = match fixture_init.unwrap().sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, t) => Some(t),
    };

    for func in test_functions {
        let Some(arg) = func.sig.inputs.get(1) else {
            return Err(parse::Error::new(
                func.sig.ident.span(),
                "Second arg in each test must be a &mut of the `#[fixture]` return type",
            ));
        };

        let does_type_match = |arg: &PatType| -> bool {
            check_fixture_type(arg.ty.clone(), fixture_type.clone().unwrap())
        };

        match arg {
            FnArg::Typed(p) if does_type_match(p) => (),
            _ => {
                return Err(parse::Error::new(
                    func.sig.ident.span(),
                    "Second arg in each test must be a &mut of the `#[fixture]` return type",
                ));
            }
        }
    }

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

    module
        .content
        .as_mut()
        .expect("Content should be present")
        .1
        .extend(parsed_items_wrapper.items);

    Ok(quote! {#module}.into())
}
