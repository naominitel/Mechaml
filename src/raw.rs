//! Raw representation of OCaml values
//!
//! This module contains types and functions to deal with the raw representation
//! of OCaml values, that is, memory words representing either a tagged, unboxed
//! integer, or an aligned pointer to the OCaml heap.
//!
//! Operations on those values are neither GC-safe nor type-safe and are
//! therefore marked as `unsafe`. They should probably not be used directly.
//! This part of the API shouldn't be considered stable either.
// TODO: Most of this could be built on top of Raml.

/// The underlying representation of every ocaml values, either
/// immediate or boxed
///
/// This corresponds to the ML value type of the C API and should never
/// been manipulated directly as it requires careful interaction with
/// the garbage collector.
// FIXME #[packed]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Value(pub isize);

/// Converts a Rust int to the ML value that represents it
///
/// ML integers are stored as immediate values with an additional type
/// bit. Because of that, ML integers are 31-bits (or 63-bits on an
/// 64-bits platform) and this macro might return a wrong result if the
/// input integer is too big.
#[macro_export] macro_rules! val_int (
    ( $i:expr ) => ( $crate::raw::Value(($i) << 1 | 1) )
);

/// Converts an ML value-encoded int to a native Rust int
///
/// Can bes Opposite operation of [`val_int`], except it takes the underlying
/// `isize` instead of the wrapping `Value`.
#[macro_export] macro_rules! int_val (
    ( $v:expr ) => ( ($v) >> 1 )
);

/// Raw allocation primitive
///
/// This is the direct (unsafe) interface to the Garbage-Collector. `alloc(tag,
/// values)` will allocate a fresh block on the OCaml heap with the given `tag`.
/// The `values` argument is an array of values which will be used to populate
/// the fields of the block. The size of the block is derived from the length of
/// this array. To create a block which fields are uninitialized, an array
/// filled with `val_int!(0)` should be used, in order for the GC not to try to
/// browse those values.
///
/// This function should probably not be used directly but instead through the
/// [`Gc`] interface.
///
/// If this function was to be called directly anyway, more information about
/// the structure and the block and how to interact safely with the GC can be
/// found here:
///
/// * [The OCaml manual, chapter 20: Interfacing with C]
/// * [Real World OCaml]
pub unsafe fn alloc(tag: u8, values: &[Value]) -> Value {
    extern "C" {
        fn caml_alloc(size: usize, tag: u8) -> *mut Value;
        fn caml_modify(tgt: *mut Value, value: Value);
    }

    unsafe {
        println!("caml_alloc({}, {})", values.len(), tag);
        let blk = caml_alloc(values.len(), tag);

        for (i, val) in values.iter().enumerate() {
            caml_modify(blk.offset(i as isize), *val);
        }

        Value(blk as isize)
    }
}
