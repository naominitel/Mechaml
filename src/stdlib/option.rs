use matching::Match;

/// Binding to the OCaml 'a option type
pub struct Option<T>(::std::marker::PhantomData<T>);

/// Lazy builder for Some() values
pub mod build {
    use mem::{Build, Gc, P};

    pub struct Some<U> {
        pub inner: U
    }

    /// Lazy builder for None values
    pub struct None<T>(pub ::std::marker::PhantomData<T>);

    unsafe impl<U> Build for Some<U> where U: Build {
        type Result = Option<U::Result>;
        fn build<'a>(self, gc: &'a mut Gc) -> &'a Option<U::Result> {
            local!{ let inner = alloc!(gc: self.inner); }
            unsafe { gc.raw_alloc(0, &[inner.value()]) }
        }
    }

    unsafe impl<T> Build for None<T> {
        type Result = Option<T>;
        fn build<'a>(self, gc: &'a mut Gc) -> &'a Option<T> {
            unsafe {
                ::std::mem::transmute(val_int!(0isize))
            }
        }
    }
}

pub fn Some<U>(of: U) -> build::Some<U> {
    build::Some { inner: of }
}

// FIXME: couldn't it be a constant instead of a function?
pub fn None<T>() -> build::None<T> {
    build::None(::std::marker::PhantomData)
}

pub mod tag {
    #[repr(isize)]
    pub enum Inline {
        Nil = 0
    }

    impl From<isize> for Inline {
        fn from(x: isize) -> Inline {
            unsafe { ::std::mem::transmute(x) }
        }
    }

    #[repr(u8)]
    pub enum Block {
        Cons = 0
    }

    impl From<u8> for Block {
        fn from(x: u8) -> Block {
            unsafe { ::std::mem::transmute(x) }
        }
    }
}

unsafe impl<'a, T: 'a> Match<'a> for Option<T> {
    type InlineTag = tag::Inline;
    type BlockTag = tag::Block;
    type BlockValue = (&'a T);
}
