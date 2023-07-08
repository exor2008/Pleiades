use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(Flush)]
pub fn pleiades_flush_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_flush(&ast)
}

fn impl_pleiades_flush(ast: &syn::DeriveInput) -> TokenStream {
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
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_into(&ast)
}

fn impl_pleiades_into(ast: &syn::DeriveInput) -> TokenStream {
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
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_pleiades_from(&ast)
}

fn impl_pleiades_from(ast: &syn::DeriveInput) -> TokenStream {
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
