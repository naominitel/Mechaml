// Re-exports
pub use stdlib::option::{Option, Some, None};
pub use stdlib::list::{List, Cons, Nil};

use std::ops::{Add, Sub, Mul, Div};
use mem::Gc;

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct int(::raw::Value);

impl ::std::fmt::Display for int {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        let &int(::raw::Value(raw)) = self;
        write!(fmt, "{}", int_val!(raw));
        Ok(())
    }
}

impl ::std::convert::From<isize> for int {
    fn from(i: isize) -> int {
        int(val_int!(i))
    }
}

unsafe impl ::mem::Build for int {
    type Result = int;

    fn build(self, gc: &mut Gc) -> &int {
        unsafe {
            ::std::mem::transmute(self)
        }
    }
}

macro_rules! impl_binop (
    ( $trait:ident, $fn:ident, $op:tt ) => (
        impl $trait for int {
            type Output = int;
            fn $fn(self, rhs: int) -> int {
                let int(::raw::Value(lhs)) = self;
                let int(::raw::Value(rhs)) = rhs;
                int(val_int!(int_val!(lhs) $op int_val!(rhs)))
            }
        }
    )
);

impl_binop!(Add, add, +);
impl_binop!(Sub, sub, -);
impl_binop!(Mul, mul, *);
impl_binop!(Div, div, /);
