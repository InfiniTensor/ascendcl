// #![doc = include_str!("../README.md")]
#![cfg(detected_ascend)]

#[macro_use]
#[allow(unused, non_upper_case_globals, non_camel_case_types, non_snake_case)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[macro_export]
    macro_rules! acl {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, 0);
        }};
    }
}

pub trait AsRaw {
    type Raw;
    /// # Safety
    ///
    /// The caller must ensure that the returned item is dropped before the original item.
    unsafe fn as_raw(&self) -> Self::Raw;
}

#[inline(always)]
pub fn init() {
    acl!(aclInit(std::ptr::null()));
}

#[inline(always)]
pub fn finalize() {
    acl!(aclFinalize());
}

#[test]
fn test_bindings() {
    init();
    finalize();
}