use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemImpl, LitInt, ReturnType, Type, parse};

enum FuncKind {
    None,
    Test,
}

#[proc_macro_attribute]
pub fn tests(args: TokenStream, input: TokenStream) -> TokenStream {
    match tests_impl(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

fn check_test_fun_sig(sig: &syn::Signature) -> bool {
    sig.constness.is_none()
        && sig.asyncness.is_none()
        && sig.unsafety.is_none()
        && sig.abi.is_none()
        && sig.generics.params.is_empty()
        && sig.generics.where_clause.is_none()
        && sig.variadic.is_none()
}

// fn check_fixture_type(arg_type: Box<Type>, fixture_type: Box<Type>) -> bool {
//     match *arg_type {
//         Type::Reference(ry) if ry.mutability.is_some() => match (*ry.elem, *fixture_type) {
//             (Type::Path(arg), Type::Path(fixture)) if arg == fixture => true,
//             _ => false,
//         },
//         _ => false,
//     }
// }

fn get_suite_type(item_impl: ItemImpl) -> Ident {
    match *item_impl.self_ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.clone(),
        _ => panic!("No"),
    }
}

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

    let mut implm: ItemImpl = syn::parse(input)?;
    let suite_ident = get_suite_type(implm.clone());
    let suite_name = suite_ident.to_string();
    let mut test_functions = Vec::new();

    for item in &mut implm.items {
        if let syn::ImplItem::Fn(func) = item {
            let mut func_kind = FuncKind::None;

            func.attrs.retain(|attr| {
                if attr.path().is_ident("test") {
                    func_kind = FuncKind::Test;
                    false
                } else {
                    true
                }
            });

            match func_kind {
                FuncKind::Test => {
                    let error = Err(parse::Error::new(
                        func.sig.ident.span(),
                        "`#[test]` function must have signature `fn(&mut self, &mut SysContext) -> Result<(), TestFailure>`",
                    ));

                    let has_return_type = match &func.sig.output {
                        ReturnType::Type(_, _) => true,
                        _ => false,
                    };

                    if func.sig.inputs.len() != 2 {
                        return error;
                    }

                    let Some(first_input) = &func.sig.inputs.first() else {
                        return error;
                    };

                    let has_self = match first_input {
                        FnArg::Receiver(r) => r.mutability.is_some() && r.reference.is_some(),
                        _ => false,
                    };

                    if !check_test_fun_sig(&func.sig) || !has_return_type || !has_self {
                        return error;
                    }

                    let ident = func.sig.ident.clone();
                    test_functions.push(ident);
                }

                _ => (),
            }
        }
    }

    let test_entries = test_functions.iter().map(|func| {
        let name = func.to_string();
        quote! {
            (
                #name,
                Box::new(|suite_dyn, context| {
                    let any_suite_dyn: &mut dyn Any = suite_dyn;
                    let suite = any_suite_dyn
                        .downcast_mut::<#suite_ident>()
                        .expect(&format!("Failed to downcast to {}", #suite_name));
                    suite.#func(context)
                }),
            )
        }
    });

    let producer_impl = quote! {

        impl oxidehtf::SuiteProducer for #suite_ident {

            fn get_suite_name(&self) -> &'static str {
                #suite_name
            }

            fn get_tests(&self) -> Vec<(&'static str, oxidehtf::DynTestFn)> {
                use std::any::Any;
                vec![#(#test_entries),*]
            }
        }
    };

    let function_name = Ident::new(
        &format!("__suite_gen_{}", suite_name),
        proc_macro2::Span::call_site(),
    );

    let register = quote! {
        fn #function_name() -> Box<dyn oxidehtf::SuiteProducer> {
            Box::new(#suite_ident::new())
        }

        inventory::submit!(oxidehtf::SuiteProducerGenerator {
            func: #function_name,
            prio: #value
        });
    };

    let expanded = quote! {
        #implm

        #producer_impl

        #register
    };

    Ok(expanded.into())
}
