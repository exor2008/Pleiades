use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse, parse_macro_input, DeriveInput, Ident, ItemEnum, Token};
use to_snake_case::ToSnakeCase;

#[proc_macro_derive(Flush)]
pub fn pleiades_flush_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_flush(&ast)
}

fn impl_pleiades_flush(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Flush for #name #ty_generics #where_clause
        {
            async fn flush(&mut self) {
                self.led.flush().await;
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(Into)]
pub fn pleiades_into_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_into(&ast)
}

fn impl_pleiades_into(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics Into<Ws2812<'a, P, S, N>> for #name #ty_generics #where_clause
        {
            fn into(self) -> Ws2812<'a, P, S, N> {
                self.led.into()
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(From)]
pub fn pleiades_from_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_from(&ast)
}

fn impl_pleiades_from(ast: &DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics From<Ws2812<'a, P, S, N>> for #name #ty_generics #where_clause
        {
            fn from(ws: Ws2812<'a, P, S, N>) -> Self {
                Self::new(ws)
            }
        }
    };
    gen.into()
}

struct Args {
    variants: Vec<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let variants = Punctuated::<Ident, Token![,]>::parse_terminated(input)?;
        Ok(Args {
            variants: variants.into_iter().collect(),
        })
    }
}

#[proc_macro_attribute]
pub fn enum_world(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as ItemEnum);
    let args = parse_macro_input!(attr as Args);
    let name = &item.ident;

    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let mut from_world_funcs = quote! {};
    let mut into_funcs = quote! {};
    let mut on_directions_funcs = quote! {};

    for variant in &args.variants {
        let snake = format_ident!("{}", variant.to_string().to_snake_case());
        let func_name = format_ident!("{}_from", snake);
        let func_code = quote! {
            pub fn #func_name (ws: Ws2812<'a, P, S, N>) -> Self {
                let #snake = #snake::#variant::from(ws);
                World::#variant(#snake)
            }
        };
        from_world_funcs.extend(func_code);

        let into_func_code = quote! {
            Self::#variant(#snake) => #snake.into(),
        };
        into_funcs.extend(into_func_code);

        let on_direction_func_code = quote! {
            Self::#variant(#snake) => #snake.on_direction(direction),
        };
        on_directions_funcs.extend(on_direction_func_code);
    }

    let gen = quote! {
        #item

        impl #impl_generics #name #ty_generics #where_clause
        {
            #from_world_funcs
        }

        impl #impl_generics Into<Ws2812<'a, P, S, N>> for #name #ty_generics #where_clause
        {
            fn into(self) -> Ws2812<'a, P, S, N> {
                match self {
                    #into_funcs
                }
            }
        }

        impl #impl_generics OnDirection for #name #ty_generics #where_clause
        {
            fn on_direction(&mut self, direction: Direction) {
                match self {
                    #on_directions_funcs
                }
            }
        }
    };

    gen.into()
}
