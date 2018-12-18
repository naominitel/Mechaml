extern crate proc_macro;
extern crate proc_macro2;
#[macro_use] extern crate quote;

#[cfg(not(feature = "unstable"))] enum Level { Error }
#[cfg(feature = "unstable")] use proc_macro::Level;

#[cfg(not(feature = "unstable"))]
fn diagnostic(_: Level, _: Span, msg: &str) {
    panic!(msg)
}

#[cfg(feature = "unstable")]
fn diagnostic(lvl: Level, sp: Span, msg: &str) {
    ::proc_macro::Diagnostic::spanned(sp.into(), lvl, msg).emit();
}

fn impl_enum(inp: DataEnum, ty: Ident) -> Result<TokenStream, ()> {
    let mut tag_boxed = 0;
    let mut tag_unboxed = 0;

    // For each constructor we generate:
    // - A lazy builder type
    // - An impl of Build for this builder type
    // - A function returning a builder from an input
    // - A corresponding destructor macro
    let ctor_tys = vec![];
    let ctor_impls = vec![];
    let ctor_fns = vec![];
    let dtor_macs = vec![];
    // The destructor macro references a variable in one of two enums depending
    // on wether the constructor is boxed or unboxed.
    let dtors_boxed = vec![];
    let dtors_unboxed = vec![];

    // generate two macros for each constructor, one for the constructor,
    // and one for the corresponding matcher.
    let macros = data_enum.variants.map(|var| {
        match var.discriminant {
            Some((Eq([span]), _)) =>
                return Err(diagnostic(Level::Error, span,
                                      "caml_type! does not support \
                                       variants with discriminants")),
            None => ()
        };

        match var.fields {
            Named(fields) => {
                let Brace(span) = fields.brace_token;
                return Err(diagnostic(Level::Error, fields.span,
                                      "caml_type! does not support \
                                       struct-like variants"));
            }

            Unnamed(fields) => {
                let params = fields.unnamed..into_iter.enumerate().map(|(i, field)| {
                    let id = Ident::new(format!("param{}", i));
                    (id, field.ty)
                });

                let fparams = params.map(|(id, ty)| {
                    quote!{ #id: #ty }
                });

                let into_exprs = params.map(|(id, _)| {
                    quote!{ id.value() }
                });

                let tts = quote!{
                    pub fn #(variant.ident)(#(fparams),+) -> ::mechaml::raw::MLValue {
                        unsafe {
                            ::mechaml::raw::alloc(#tag_boxed, &[#(into_exprs),+]);
                        }
                    }

                    // TODO: pattern macro
                    // macro_rules! #(variant.ident)(
                    //     #()
                    // )
                }

                tag_boxed += 1;
                tts
            }

            Unit => {
                let tts = quote!{
                    pub fn #(variant.ident)() -> ::mechaml::raw::MLValue {
                        (#tag_unboxed << 1 ) & 1
                    }
                };
                tag_unboxed += 1;
                tts
            }
        };
    });

    in quote! { #( #macros )+ }
}

#[proc_macro_derive(MechamlEnum)]
pub fn caml_type(inp: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let inp: syn::DeriveInput = syn::parse().unwrap();

    // FIXME: what about attributes?
    let typedef = quote!{
        #(inp.visiblity) struct #(inp.ident)<#(inp.generics)>(::mechaml::rawMLValue);
    }

    match inp.data {
        Struct(data_struct) => impl_record(data_struct),
        Enum(data_enum) => impl_enum(data_enum),
    }
}
