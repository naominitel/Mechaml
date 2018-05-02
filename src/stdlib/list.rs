
#[macro_use] use macros;
use matching::Match;

/// Binding to the OCaml 'a list type
pub struct List<T>(::std::marker::PhantomData<T>);

pub mod build {
    use mem::{Build, Gc, P};
    use super::List;

    pub struct Cons<U, V> {
        pub hd: U,
        pub tl: V
    }

    pub struct Nil<T>(pub ::std::marker::PhantomData<T>);

    unsafe impl<U, V> Build for Cons<U, V>
        where U: Build,
            V: Build<Result = List<U::Result>> {
        type Result = List<U::Result>;

        fn build(self, gc: &mut Gc) -> &List<U::Result> {
            local! {
                let hd: P<U::Result> = alloc!(gc: self.hd);
                let tl: P<V::Result> = alloc!(gc: self.tl);
            }

            unsafe { gc.raw_alloc(0, &[hd.value(), tl.value()]) }
        }
    }

    unsafe impl<T> Build for Nil<T> {
        type Result = List<T>;

        fn build(self, at: &mut Gc) -> &List<T> {
            unsafe { ::std::mem::transmute(val_int!(0))}
        }
    }
}

pub fn Cons<U, V>(hd: U, tl: V) -> build::Cons<U, V> {
    build::Cons { hd, tl }
}

// FIXME: couldn't this be a constand instead of a function?
pub fn Nil<T>() -> build::Nil<T> {
    build::Nil(::std::marker::PhantomData)
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

unsafe impl<'a, T: 'a> Match<'a> for List<T> {
    type InlineTag = tag::Inline;
    type BlockTag = tag::Block;
    type BlockValue = (T, &'a List<T>);
}

#[macro_export] macro_rules! list {
    [] => {
        $crate::matching::Matcher::Inline($crate::stdlib::list::tag::Inline::Nil)
    } ;
    ($hd:ident :: $tl:ident) => {
        $crate::matching::Matcher::Block(
            $crate::stdlib::list::tag::Block::Cons,
            &($hd, $tl)
        )
    }
}
