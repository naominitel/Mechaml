//! Safe, ergonomic macros.
//!
//! The macros contained in this module make using the internal APIs of this
//! library easily and without any explicit use of `unsafe`. Though those
//! macros perform `unsafe` calls internally, code using them to access the ML
//! APIs should be considered safe.

//! Declares local variables as GC roots.
//!
//! This macro is the primary safe way to declare one or more local variables
//! referencing OCaml data.
//!
//! ```
//! local!{
//!     let x = alloc!(gc: Some(Some(None)));
//!     let y = alloc!(gc: Cons(1, Cons(2, Cons(3, Nil))));
//! }
//! ```
//!
//! Variables declared with this macro will act as a GC root: as long as the
//! variable is in scope, the referenced data wll be considered reachable by the
//! OCaml GC and is guaranteed not to be freed. The memory is not guaranteed to
//! be freed once the variable goes out of scope, however, as other roots
//! pointing to the same data might still exist, either in the Rust or the OCaml
//! world.
//!
//! The variables are therefore expected to by of type `P<T>` where T is the
//! OCaml type. See the documentation of the [`P`] type for more information
//! about how to use the defined variables, and the management of GC roots.
//!
//! The initializing expression is expected to have type &T, where T is the
//! underlying OCaml type. Once the local root as been created, the reference
//! can be safely dropped, releasing the GC for future allocations.
//!
//! The type should be inferred by the compiler in most cases, but the explicit
//! type annotation syntax is available just like standard let-bindings.
//!
//! Note: destructuring pattern-binding (as in `let (x, y) = ... ;` is currently
//! not supported), but might be in the future.
// FIXME:
// * patterns
// * repetition of type annotations. I didn't find any better for now.
//   it accepts the desired syntax but will produce weird error messages for
//   errorneous inputs like let x: t1: t2 = ...
#[macro_export] macro_rules! local {
    { $( let $binder:ident $( : $ty:ty )* = $e:expr ; )+ } => {
        $(
            let $binder $( : $ty )*;
            unsafe {
                ($binder) = $crate::mem::P::new();
                ($binder).register(); // This value is register to the GC and cannot be moved
                ($binder).root($e);
            }
        )+
    }
}

/// Declares an OCaml primitive.
///
/// This macro defines a wrapper around a function, to be used as an entry point
/// from the OCaml world. This macro should always be used instead of defining
/// entry points manually to make sure that the handling of memory management is
/// safe. Example:
///
/// ```
/// ml_extern! {
///     fn caml_count(list: List<int>) -> int = count;
/// }
/// ```
///
/// This will create a function accessible from the OCaml side with the
/// following signature:
///
/// ```
/// external count : int list -> int = "caml_count"
/// ```
///
/// On the Rust side, a safe function `count` is expected to exist to provide
/// the actual implementation:
///
/// ```
/// fn count(gc: &'a Gc, list: &'a List<int>) -> &'a int {
///     // Your implementation...
/// }
/// ```
///
/// Notice that, compared to the primitive, the implementation function:
///
/// * Is neither `unsafe`, nor `extern`, nor `#[no_mangle]`, etc. It's a plain
///   Rust function.
/// * Takes an additional argument of type `Gc` to be able to perform
///   allocations.
/// * Takes the other arguments as references bound by the lifetime of the `Gc`.
///   Those values are rooted and can be used safely for the whole duration of
///   the call. Likewise, the return value is required to be a reference with
///   the same validity (which will happen if it's allocated through the given
///   `Gc`).
///
/// Note: The `count` name on the Rust side is the name of the implementation
/// function. The `count` name on the OCaml side is the name that will be used
/// to call the `external`. They do not have to match. The name of the wrapping
/// primitive, `caml_count` in the example above, is required to match or the
/// program won't link properly. The name of the implementation function could
/// even be a path to a function in another module:
///
/// ```
/// ml_extern! {
///     fn caml_count(list: List<int>) -> int = ::some::module::count;
/// }
/// ```
#[macro_export] macro_rules! ml_extern {
    ( $( fn $caml_name:ident ( $($arg_id:ident : $arg_ty:ty),* )
                               -> $ret_ty:ty = $rust_fn:path; )* ) => {
        $(
            #[no_mangle]
            pub unsafe extern "C" fn $caml_name($($arg_id: $crate::raw::Value),*)
                                                -> $crate::raw::Value {
                $(
                    let $arg_id: $crate::mem::P<$arg_ty> =
                        $crate::mem::P::from($arg_id);
                    ($arg_id).register();
                ),*

                let mut gc = $crate::mem::Gc::new();
                let ret: &$ret_ty = $rust_fn (&mut gc, $($arg_id.as_ref()),*);

                ::std::mem::transmute(ret)
            }
        )*
    }
}
