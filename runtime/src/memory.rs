use crate::{
    bindings::{aclrtMemMallocPolicy::ACL_MEM_MALLOC_HUGE_FIRST, aclrtMemcpyKind::*},
    CurrentCtx, Stream,
};
use context_spore::{impl_spore, AsRaw};
use std::{
    alloc::Layout,
    ffi::c_void,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::null_mut,
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(transparent)]
pub struct DevByte(#[allow(unused)] u8);

#[inline]
pub fn memcpy_d2h<T: Copy>(dst: &mut [T], src: &[DevByte]) {
    let len = size_of_val(dst);
    let dst = dst.as_mut_ptr().cast();
    assert_eq!(len, size_of_val(src));
    acl!(aclrtMemcpy(
        dst,
        len,
        src.as_ptr().cast(),
        len,
        ACL_MEMCPY_DEVICE_TO_HOST,
    ));
}

#[inline]
pub fn memcpy_h2d<T: Copy>(dst: &mut [DevByte], src: &[T]) {
    let len = size_of_val(src);
    let src = src.as_ptr().cast();
    assert_eq!(len, size_of_val(dst));
    acl!(aclrtMemcpy(
        dst.as_mut_ptr().cast(),
        len,
        src,
        len,
        ACL_MEMCPY_HOST_TO_DEVICE,
    ));
}

#[inline]
pub fn memcpy_d2d(dst: &mut [DevByte], src: &[DevByte]) {
    let len = size_of_val(src);
    assert_eq!(len, size_of_val(dst));
    acl!(aclrtMemcpy(
        dst.as_mut_ptr().cast(),
        len,
        src.as_ptr().cast(),
        len,
        ACL_MEMCPY_DEVICE_TO_DEVICE,
    ));
}

impl Stream<'_> {
    #[inline]
    pub fn memcpy_h2d<T: Copy>(&self, dst: &mut [DevByte], src: &[T]) {
        let len = size_of_val(src);
        assert_eq!(len, size_of_val(dst));
        acl!(aclrtMemcpyAsync(
            dst.as_mut_ptr().cast(),
            len,
            src.as_ptr().cast(),
            len,
            ACL_MEMCPY_HOST_TO_DEVICE,
            self.as_raw(),
        ));
    }

    #[inline]
    pub fn memcpy_d2d(&self, dst: &mut [DevByte], src: &[DevByte]) {
        let len = size_of_val(src);
        assert_eq!(len, size_of_val(dst));
        acl!(aclrtMemcpyAsync(
            dst.as_mut_ptr().cast(),
            len,
            src.as_ptr().cast(),
            len,
            ACL_MEMCPY_DEVICE_TO_DEVICE,
            self.as_raw(),
        ));
    }
}

struct Blob<P> {
    ptr: P,
    len: usize,
}

impl_spore!(DevMem and DevMemSpore by (CurrentCtx, Blob<*mut c_void>));

impl CurrentCtx {
    pub fn malloc<T: Copy>(&self, len: usize) -> DevMem<'_> {
        let len = Layout::array::<T>(len).unwrap().size();
        let mut ptr = null_mut();
        // NOTICE 8.0.RC3.alpha1 只有 ACL_MEM_MALLOC_HUGE_FIRST 有效
        acl!(aclrtMalloc(&mut ptr, len, ACL_MEM_MALLOC_HUGE_FIRST));
        DevMem(unsafe { self.wrap_raw(Blob { ptr, len }) }, PhantomData)
    }

    pub fn from_host<T: Copy>(&self, slice: &[T]) -> DevMem<'_> {
        let len = size_of_val(slice);
        let src = slice.as_ptr().cast();
        let mut ptr = null_mut();
        acl!(aclrtMalloc(&mut ptr, len, ACL_MEM_MALLOC_HUGE_FIRST));
        acl!(aclrtMemcpy(ptr, len, src, len, ACL_MEMCPY_HOST_TO_DEVICE));
        DevMem(unsafe { self.wrap_raw(Blob { ptr, len }) }, PhantomData)
    }
}

impl Drop for DevMem<'_> {
    #[inline]
    fn drop(&mut self) {
        acl!(aclrtFree(self.0.rss.ptr));
    }
}

impl Deref for DevMem<'_> {
    type Target = [DevByte];
    #[inline]
    fn deref(&self) -> &Self::Target {
        if self.0.rss.len == 0 {
            &[]
        } else {
            unsafe { from_raw_parts(self.0.rss.ptr as _, self.0.rss.len) }
        }
    }
}

impl DerefMut for DevMem<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        if self.0.rss.len == 0 {
            &mut []
        } else {
            unsafe { from_raw_parts_mut(self.0.rss.ptr as _, self.0.rss.len) }
        }
    }
}

impl AsRaw for DevMemSpore {
    type Raw = *mut c_void;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss.ptr
    }
}

impl DevMemSpore {
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.rss.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.rss.len == 0
    }
}
