use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Lit, Meta, MetaNameValue};

#[proc_macro_derive(Confu, attributes(confu_prefix, default, protect, hide))]
pub fn confu_macro_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    println!("{:#?}", input);
    let name = &input.ident;
    let mut prefix = String::from("");
    if input.attrs.len() == 1 {
        let meta = input.attrs[0].parse_meta().unwrap();
        if meta.path().is_ident("confu_prefix") {
            if let Meta::NameValue(MetaNameValue { ref lit, .. }) = meta {
                if let Lit::Str(s) = lit {
                    prefix = s.value();
                }
            }
        }
    }

    let mut resolvers: Vec<(&Ident, proc_macro2::TokenStream)> = Vec::new();
    let mut printers: Vec<proc_macro2::TokenStream> = Vec::new();

    // get at the struct data
    if let syn::Data::Struct(ds) = &input.data {
        // get at the named struct fields
        if let syn::Fields::Named(fields) = &ds.fields {
            for field in fields.named.iter() {
                if let Some(ident) = &field.ident {
                    let env_var_name = format!(
                        "{}{}",
                        prefix.to_uppercase(),
                        ident.to_string().to_uppercase()
                    );

                    // process attributes if such are present
                    let mut do_require = false;
                    let mut do_protect = false;
                    let mut do_hide = false;
                    let mut has_default = false;
                    let mut default = String::from("");

                    for attr in field.attrs.iter() {
                        let meta = attr.parse_meta().unwrap();

                        match meta.path().get_ident() {
                            Some(ident) if ident == "require" => {
                                println!("{}: found `require` attr", env_var_name);
                                do_require = true;
                            }
                            Some(ident) if ident == "default" => {
                                println!("{}: found `default` attr", env_var_name);
                                if let Meta::NameValue(MetaNameValue { ref lit, .. }) = meta {
                                    if let Lit::Str(s) = lit {
                                        default = s.value();
                                        has_default = true;
                                    }
                                }
                            }
                            Some(ident) if ident == "hide" => {
                                println!("{}: found `hide` attr", env_var_name);
                                do_hide = true;
                            }
                            Some(ident) if ident == "protect" => {
                                println!("{}: found `protect` attr", env_var_name);
                                do_protect = true;
                            }
                            Some(ident) => panic!("unsupported attribute {}", ident),
                            None => panic!("unsupported attribute"),
                        }
                    }

                    resolvers.push((
                        ident,
                        quote_resolver(
                            &env_var_name,
                            do_require,
                            // do_protect,
                            // do_hide,
                            has_default,
                            &default,
                        ),
                    ));

                    if !do_hide {
                        printers.push(quote_printer(&ident, &env_var_name, do_require, do_protect));
                    }
                }
            } // for each field
        } // in named fields
    } // in struct data

    let expanded: proc_macro2::TokenStream = quote! {
        impl Confu for #name {
            fn confu() {
                println!("Confu is not yet implemented for struct {} with prefix '{}'", stringify!(#name), #prefix);
            }
        }
    };
    TokenStream::from(expanded)
}

fn quote_resolver(
    key: &str,
    is_required: bool,
    // is_protected: bool,
    // is_hidden: bool,
    has_default: bool,
    default: &str,
) -> proc_macro2::TokenStream {
    quote! {{
        // first see if we have a runtime argument provided
        let maybe_from_args = std::env::args().skip(1).find_map(|arg| {
            if let Some((k, v)) = arg.trim_matches('-').split_once('=') {
                if k == #key.to_lowercase() {
                    Some(String::from(v))
                } else {
                    None
                }
            } else {
                None
            }
        });

        // if argument was provided as a runtime argument, use it,
        // otherwise, see if a corresponding environment variable is set,
        // finally use default if provided.
        let maybe = match maybe_from_args {
            Some(val) => Some(val),
            None => {
                let maybe_from_env = env::var(#key);
                match maybe_from_env {
                    Ok(val) => Some(val),
                    None => {
                        if #has_default {
                            Some(String::from(#default))
                        } else {
                            None
                        }
                    } ,
                }
            }
        };

        // 1. return a resulting argument if we were able to find it one way
        //    or the other, or an empty string.
        // 2. return an error if argument was required but was not provided
        match maybe {
            Some(val) => Ok(val),
            None => {
                if !#is_required {
                    Ok(String::from(""))
                } else {
                    Err(confu::ConfuError::MissingRequired(format!("required argument {} was not provided.", #key)))
                }
            }
        }

    }}
}

fn quote_printer(
    ident: &Ident,
    key: &str,
    is_required: bool,
    is_protected: bool,
) -> proc_macro2::TokenStream {
    quote! {}
}
// KEEP SAKE
// let mut prefix: Option<String> = None;
// for attr in &input.attrs {
//     let meta = attr.parse_meta().unwrap();
//     // println!("attr name: {:?}", attr.name());
//     match &meta {
//         Meta::NameValue(nv) => {
//             // println!("attr: {:#?}", attr);

//             println!("attr: {:?}", meta.path().is_ident("confu_prefix"));
//             if meta.path().is_ident("confu_prefix") {
//                 if let Lit::Str(lstr) = &nv.lit {
//                     prefix = Option::Some(lstr.value());
//                 }
//             }
//             println!("attr: {:?}", nv.lit);
//         }
//         // Meta::NameValue(attr) => {
//         //     // println!("attr: {:#?}", attr);
//         //     println!("attr: {:?}", attr.path);
//         // }
//         _ => {
//             println!("unexpected attr: {:#?}", meta);
//         }
//     }
// }
