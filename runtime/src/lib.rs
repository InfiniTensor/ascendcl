#![doc = include_str!("../README.md")]
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
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, 0);
        }};
    }
}

#[inline(always)]
pub fn init() {
    acl!(aclInit(std::ptr::null()));
}

#[inline(always)]
pub fn finalize() {
    acl!(aclFinalize());
}

#[inline]
pub fn version() -> (i32, i32, i32) {
    let mut ans = (0, 0, 0);
    acl!(aclrtGetVersion(&mut ans.0, &mut ans.1, &mut ans.2));
    ans
}

mod context;
mod dev_mem;
mod device;
mod event;
mod host_mem;
mod stream;

pub use context::{Context, CurrentCtx, NoCtxError};
pub use context_spore::{impl_spore, AsRaw, ContextResource, ContextSpore, RawContainer};
pub use dev_mem::{memcpy_d2d, memcpy_d2h, memcpy_h2d, DevByte, DevMem, DevMemSpore};
pub use device::Device;
pub use event::{Event, EventSpore};
pub use host_mem::{HostMem, HostMemSpore};
pub use stream::{Stream, StreamSpore};

struct Blob<P> {
    ptr: P,
    len: usize,
}

#[test]
fn test_bindings() {
    init();
    println!("version: {:?}", version());
    finalize()
}
