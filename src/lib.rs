// FIXME
#![allow(unused)]

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

#[macro_use] pub mod raw;
#[macro_use] pub mod mem;
#[macro_use] pub mod macros;
#[macro_use] pub mod matching;
#[macro_use] pub mod stdlib;

// Tests in this module will only test the API and never actually run when
// building with `cargo test`.
// To run a complete test, they can be compiled and linked with the provided
// `test.ml` file the following way:
//    rustc -g --emit link --crate-type staticlib --cfg test -o libtest.a src/lib.rs
//    ocaml -g -custom -o test tests/test.ml libtest.a

#[cfg(test)] pub mod map {
    use mem::{Gc, P};
    use matching::match_;
    use stdlib::pervasives::*;

    ml_extern! {
        fn caml_map(lst: List<int>) -> List<int> = map;
    }

    fn map<'a>(gc: &'a mut Gc, lst: &'a List<int>) -> &'a List<int> {
        match match_(lst) {
            list![] => lst,
            list![hd :: tl] => {
                local!{ let rec = map(gc, tl); }
                alloc!(gc: Cons(hd + int::from(1), rec.as_ref()))
            }
        }
    }
}

#[cfg(test)] pub mod test{
    use raw;
    use mem::{Gc, P};
    use stdlib::pervasives::*;

    #[no_mangle]
    pub extern "C" fn foo(x: raw::Value) -> raw::Value {
        let ref mut gc = unsafe { Gc::new() };

        local!{
            let x = alloc!(gc: Some(Some(Some(int::from(43)))));
        }

        unsafe {
            x.value()
        }
    }
}
