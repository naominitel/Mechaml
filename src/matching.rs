//! Pattern-matching on OCaml values
//!
//! This module contains types and functions to simulate pattern-matching on
//! OCaml values.
//!
//! Though most of those functions are safe to use, they can be used more
//! ergonomicly through the matching macros generated for each OCaml type.
//!
//! Matching on OCaml values typically involves converting the value into a
//! [`Matcher`] item, which can be examined with appropriate patterns (typically
//! constructed by macros). Such conversions are shallow and cheap
//! (non-copying):
//!
//! ```
//! local! {
//!     let value = alloc!{ gc | Cons(1, Cons(2, Cons(3, Nil))) };
//! }
//!
//! match value.to() {
//!     list![] => ... ,              // The value is Nil
//!     list![ hd :: tl ] => ... ,    // The value is a Cons of hd and tl
//! }
//! ```
//!
//! Unfortunately, nested patterns aren't allowed yet with this mechanism, as it
//! would require either being able to call `.to()` on subpatterns, are to
//! deeply-convert the structure (which is costly). This might evolve in the
//! future if Rust ever has automatic calls to AsRef or Deref in
//! pattern-matchings, but in the meantime, nesting `match` blocks will be
//! required to deeply-inspect a structure.

#[macro_use] use raw;

/// A type allowing inspection of an OCaml value.
///
/// This type ‶describes″ the content of an OCaml value. A typical OCaml value
/// (enum, record, ...) is represented either as an unboxed integer or as a
/// boxed block containing a tag and fields.
pub enum Matcher<'a, T> where T: Match<'a>, T::BlockValue: 'a {
    Inline(T::InlineTag),
    Block(T::BlockTag, &'a T::BlockValue)
}

pub fn match_<'a, T: Match<'a>>(val: &'a T) -> Matcher<'a, T> {
    let raw::Value(raw) = unsafe { ::std::mem::transmute(val) };
    if raw & 1 == 0 {
        // Pointer
        unsafe {
            let hdr_p = ::std::mem::transmute::<_, *mut usize>(raw).offset(-1);
            let tag = T::BlockTag::from(*(hdr_p as *mut u8));
            let data_p = ::std::mem::transmute(raw);
            Matcher::Block(tag, data_p)
        }
    } else {
        // unboxed tag
        Matcher::Inline(T::InlineTag::from(int_val!(raw)))
    }
}

pub unsafe trait Match<'a> {
    type InlineTag: From<isize>;
    type BlockTag: From<u8>;
    type BlockValue;
}
