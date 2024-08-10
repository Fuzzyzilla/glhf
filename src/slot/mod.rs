//! Initialization, modification, and access to GL objects.

pub mod marker;

pub mod buffer;
pub mod framebuffer;
pub mod program;
pub mod texture;
pub mod vertex_array;

/// create a reference to a ZST out of thin air for the given lifetime
fn zst_mut<'a, T>() -> &'a mut T {
    const {
        assert!(std::mem::size_of::<T>() == 0);
    };

    // Use an arbitrary pointer. ZSTs do not require a valid allocated object,
    // but they *do* require a valid (well-aligned and non-null) address.
    let mut dummy_ptr = std::ptr::NonNull::<T>::dangling();

    unsafe { dummy_ptr.as_mut() }
}
/// create a reference to a ZST out of thin air for the given lifetime
fn zst_ref<'a, T>() -> &'a T {
    const {
        assert!(std::mem::size_of::<T>() == 0);
    };

    // Use an arbitrary pointer. ZSTs do not require a valid allocated object,
    // but they *do* require a valid (well-aligned and non-null) address.
    let dummy_ptr = std::ptr::NonNull::<T>::dangling();

    unsafe { dummy_ptr.as_ref() }
}
