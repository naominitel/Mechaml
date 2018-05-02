use std::cell::UnsafeCell;
#[macro_use] use raw;

/// The trait of types that can build into an ML value.
pub unsafe trait Build {
    type Result;

    /// Creates the value represented by self, allocating the necessary
    /// blocks on the OCaml heap, and stores the resulting root pointer
    /// in `at`.
    ///
    /// The pointer in `at` should have already been registered before
    /// calling this function on it.
    fn build<'a>(self, &'a mut Gc) -> &'a Self::Result;
}

/// The Garbage-Collector interface
///
/// This structs wraps over the ML garbage-collected dynamic memory management.
/// All accesses to the OCaml memory management engine must go through this
/// struct in order to ensure that the manipulation of OCaml values from Rust is
/// memory safe.
///
/// Memory safety here means that OCaml values visible only to the Rust code are
/// registered as roots to the OCaml GC whenever a sensible operation is to be
/// peformed. This ensures that:
///
/// * The OCaml GC won't release memory that is still accessible by the Rust
/// code (which would cause memory corruption).
///
/// * Such values will be freed once they're not accessible by the Rust code
/// anymore (unless they are from OCaml code).
///
/// This is done by having all memory-sensible operations go through the [`Gc`]
/// type which will be borrowed whenever unregistered values exist, preventing
/// any other call of such an operation until the value as been rooted.
///
/// Sensible memory-operations mainly include heap-allocating values, which is
/// performed by the [`alloc`] and [`raw_alloc`] methods, but will mainly be
/// done more transparently through the [`alloc`] macro.
// NOTE: This is a tupe-struct around unit just to make it an abstract type and
// prevent new values from being called without using the unsafe `new` method.
pub struct Gc(());

impl Gc {
    pub unsafe fn new() -> Gc {
        Gc(())
    }

    pub unsafe fn raw_alloc<'a, T>(&self, tag: u8, fields: &[raw::Value]) -> &'a T {
        ::std::mem::transmute(raw::alloc(tag, fields))
    }
}

#[macro_export] macro_rules! alloc {
    ($gc:ident : $e:expr) => {{ use $crate::mem::Build; ($e).build($gc) }}
}

// FIXME #[packed]
/// A garbage-collected ML pointer.
///
/// This type is a smart-pointer representing a heap-allocated OCaml value which
/// enforces that this value is registered to the OCaml garbage collector before
/// anything can be done with it.
///
/// This can be seen as a ‶root″ of the GC. It's stack location is registered as
/// a root to the GC when first created, it can then be copied to other roots,
/// and is unregistered once the value is destroyed. The GC will then be able to
/// collect the underlying value, unless other roots pointing to the same
/// location are still registered somewhere.
///
/// As a result of their stack location being registered, those values are
/// ‶locked″ in place once they have been initialized: they cannot be moved, but
/// can be cheaply cloned or converted to lifetime-bound lightweight references
/// which can be easily passed around.
///
/// Those values are memory-safe and type-safe but most operations on them are
/// not, as they directly deal with the underlying value, which could point to
/// data of any OCaml type. Those functions are thus marked `unsafe` and
/// shouldn't be directly used. This type should be used only through the
/// higher-level functions and macros provided for each actual OCaml type (like
/// Option, List, etc.), as well as the [`local`] and [`alloc`] macros.
pub struct P<'a, T: 'a> {
    val: UnsafeCell<raw::Value>,
    root: UnsafeCell<CamlRootsBlock>,
    // This has to be an UnsafeCell inside the PhantomData for some reason I'm
    // not sure to understand. Using simply &'a T messes with the lifetime
    // guarantees of this type.
    marker: ::std::marker::PhantomData<UnsafeCell<&'a T>>
}

// Private: a chunk of stack data representing a root of the GC.
// It contains the location of the referenced values (in our case, always one),
// and a linked pointer to the next root (which must be restored) when this one
// goes out of scope.
//
// We always store a single value, so ntables == nitems == 1 at any time.
// tables[0] contains the root location.
#[repr(C)]
pub struct CamlRootsBlock {
    pub next: *mut CamlRootsBlock,
    pub ntables: usize,
    pub nitems: usize,
    pub tables: [*mut raw::Value; 1],
}

extern "C" {
    // The main OCaml runtime global variable which points to the first linked
    // node of GC roots (that is, the latest root).
    pub static mut caml_local_roots: *mut CamlRootsBlock;
}

impl Drop for CamlRootsBlock {
    fn drop(&mut self) {
        // debug!("unregistering local root {:?}", self as *mut CamlRootsBlock);
        unsafe { caml_local_roots = self.next };
    }
}

impl<'a, T: 'a> P<'a, T> {
    /// Create a new, uninitialized pointer.
    ///
    /// This function is unsafe as passing the resulting value to code expecting
    /// a valid pointer will fail. Any attempt to intialize this pointer before
    /// it's registered might cause the GC to release the still-accessible
    /// pointed memory.
    ///
    /// This function should probably not be used directly, but rather through
    /// the [`local`] macro.
    // NOTE: This value is initialized with an OCaml value representing the
    // integer 1. Because of that, any use of that value will cause undefined
    // behaviour but the GC will be able to safely browse it, if a cycle occurs
    // after it is registered but before it is initialized.
    pub unsafe fn new() -> P<'a, T> {
        P {
            val: UnsafeCell::new(val_int!(0)),
            root: UnsafeCell::new(CamlRootsBlock {
                ntables: 0,
                nitems: 0,
                tables: [::std::ptr::null_mut()],
                next: ::std::ptr::null_mut()
            }),
            marker: ::std::marker::PhantomData
        }
    }

    /// Registers this value to the garbage-collector.
    ///
    /// Because of the `'a` requirement on self, calling this method will make
    /// the value ‶locked″ in place, preventing it from being moved or copied
    /// without explicit cloning.
    ///
    /// This function is unsafe because attempting to register a single pointer
    /// multiple times might cause leakage of unaccessible memory.
    ///
    /// This function should probably not be used directly, but rather through
    /// the [`local`] macro.
    pub unsafe fn register(&'a self) {
        // debug!("registering location {:?} as GC root {:?}",
        //        self.val.get(),
        //        self.root.get());
        (*self.root.get()).nitems = 1;
        (*self.root.get()).ntables = 1;
        (*self.root.get()).tables[0] = self.val.get();
        (*self.root.get()).next = caml_local_roots;
        caml_local_roots = self.root.get();
    }

    /// Initializes this pointer through a freshly-allocated value.
    ///
    /// The input reference can then be safely dropped since the pointed value
    /// is now accessible through a registered root, thus releasing the GC for
    /// other allocation jobs.
    ///
    /// This function should probably not be used directly, but rather through
    /// the [`local`] macro.
    pub fn root<'b>(&self, val: &'b T) {
        unsafe { *self.val.get() = ::std::mem::transmute(val) }
    }

    /// Creates a new pointer from a raw underlying ML value.
    ///
    /// This is extremely unsafe as there is no guarantee that the input ML
    /// value is a valid pointer referencing valid data of the correct type.
    ///
    /// The resulting pointer value must then be registered before the GC can be
    /// used again.
    ///
    /// This function should probably not be used directly, but is used
    /// internally by the [`ml_extern`] macro.
    pub unsafe fn from(val: raw::Value) -> P<'a, T> {
        let ptr = P::new();
        *ptr.val.get() = val;
        ptr
    }

    /// Extracts the raw, underlying ML value.
    ///
    /// This function should probably not be used directly, but might be used
    /// internally by some macros or functions.
    pub unsafe fn value(&self) -> raw::Value {
        *self.val.get()
    }
}

// Enables creating bound cheap references from a rooted pointer.
//
// Those references can be passed around just like normal Rust references. Since
// they're bound to the lifetime of the root pointer, they will prevent the
// underlying data from being collected as long as they are in scope.
impl<'a, T> ::std::convert::AsRef<T> for P<'a, T> {
    fn as_ref(&self) -> &T {
        unsafe {
            let raw::Value(ptr) = self.value();
            ::std::mem::transmute::<_, &T>(ptr)
        }
    }
}

// todo: remove that! this is only safe for ml values. there should be a trait bound.
unsafe impl<'a, T> Build for &'a T {
    type Result = T;
    fn build<'b>(self, gc: &mut Gc) -> &T {
        unsafe { ::std::mem::transmute(self) }
    }
}
