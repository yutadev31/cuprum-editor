use heck::ToUpperCamelCase;
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, Ident, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
};

struct Method {
    name: Ident,
    args: Vec<(Ident, Type)>,
    ret: Option<Type>,
}

struct ApiDef {
    methods: Vec<Method>,
}

impl Parse for ApiDef {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut methods = Vec::new();

        while !input.is_empty() {
            input.parse::<Token![fn]>()?;
            let name: Ident = input.parse()?;

            let args_content;
            parenthesized!(args_content in input);

            let mut args = Vec::new();
            while !args_content.is_empty() {
                let arg_name: Ident = args_content.parse()?;
                args_content.parse::<Token![:]>()?;
                let arg_type: Type = args_content.parse()?;

                args.push((arg_name, arg_type));

                if args_content.peek(Token![,]) {
                    args_content.parse::<Token![,]>()?;
                } else {
                    break;
                }
            }

            let ret = if input.peek(Token![->]) {
                input.parse::<Token![->]>()?;
                let ret: Type = input.parse()?;
                Some(ret)
            } else {
                None
            };

            methods.push(Method { name, args, ret });
        }

        Ok(ApiDef { methods })
    }
}

#[proc_macro]
pub fn define_api(input: TokenStream) -> TokenStream {
    let ApiDef { methods } = parse_macro_input!(input as ApiDef);

    let methods_impl = methods.iter().map(|method| {
        let method_name = &method.name;
        let method_args = &method.args;
        let method_ret = &method.ret;

        let method_args = method_args.iter().map(|arg| {
            let arg_name = &arg.0;
            let arg_type = &arg.1;

            (
                quote! {
                    #arg_name: #arg_type
                },
                quote! {
                    #arg_name
                }
            )
        });

        let method_args_with_type = method_args.clone().map(|arg| arg.0);
        let method_args = method_args.map(|arg| arg.1);

        let method_camel_name = method_name.to_string().to_upper_camel_case();
        let method_camel_name = Ident::new(&method_camel_name, Span::call_site().into());

        if let Some(method_ret) = method_ret {
            quote! {
                pub async fn #method_name(&mut self, #( #method_args_with_type ),* ) -> anyhow::Result<#method_ret> {
                    if let Some(CuprumApiResponseKind::#method_camel_name(result)) = self
                        .provider
                        .send_message(CuprumApiRequestKind::#method_camel_name( #( #method_args ),* ))
                        .await?
                    {
                        Ok(result)
                    } else {
                        Err(anyhow::anyhow!("mismatched types"))
                    }
                }
            }
        } else {
            quote! {
                pub async fn #method_name(&mut self, #( #method_args_with_type ),* ) -> anyhow::Result<()> {
                    self.provider
                        .send_message(CuprumApiRequestKind::#method_camel_name( #( #method_args ),*))
                        .await?;
                    Ok(())
                }
            }
        }
    });

    let methods_enums = methods.iter().map(|method| {
        let method_camel_name = method.name.to_string().to_upper_camel_case();
        let method_camel_name = Ident::new(&method_camel_name, Span::call_site().into());

        let method_args = method.args.iter().map(|arg| {
            let arg_type = &arg.1;
            quote! {
                #arg_type
            }
        });

        (
            quote! {
                #method_camel_name( #( #method_args ),* )
            },
            (method.ret.clone()).map(|method_ret| {
                quote! {
                    #method_camel_name( #method_ret )
                }
            }),
        )
    });

    let request = methods_enums.clone().map(|method_enums| method_enums.0);
    let response = methods_enums.filter_map(|method_enums| method_enums.1);

    let enum_derive_attr: Attribute = parse_quote!(#[derive(Debug, Clone, Serialize, Deserialize)]);
    let struct_derive_attr: Attribute =
        parse_quote!(#[derive(Debug, Clone, Serialize, Deserialize)]);
    let api_struct_derive_attr: Attribute = parse_quote!(#[derive(Debug, Default)]);

    let expanded = quote! {
        #enum_derive_attr
        pub enum CuprumApiRequestKind {
            #( #request ),*
        }

        #struct_derive_attr
        pub struct CuprumApiRequest {
            pub id: RequestId,
            pub kind: CuprumApiRequestKind,
        }

        #enum_derive_attr
        pub enum CuprumApiResponseKind {
            #( #response ),*
        }

        #struct_derive_attr
        pub struct CuprumApiResponse {
            pub id: RequestId,
            pub kind: Option<CuprumApiResponseKind>,
        }

        #api_struct_derive_attr
        pub struct CuprumApi<T: CuprumApiProvider> {
            pub provider: T,
        }

        impl<T: CuprumApiProvider> CuprumApi<T> {
            pub fn new(provider: T) -> Self {
                Self { provider }
            }

            #( #methods_impl )*
        }
    };

    expanded.into()
}
