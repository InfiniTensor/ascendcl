use crate::{
    bindings::{aclrtStream, aclrtStreamStatus::*},
    CurrentCtx,
};
use context_spore::{impl_spore, AsRaw};
use std::{marker::PhantomData, ptr::null_mut};

impl_spore!(Stream and StreamSpore by (CurrentCtx, aclrtStream));

impl CurrentCtx {
    #[inline]
    pub fn stream(&self) -> Stream {
        let mut stream = null_mut();
        acl!(aclrtCreateStream(&mut stream));
        Stream(unsafe { self.wrap_raw(stream) }, PhantomData)
    }
}

impl Drop for Stream<'_> {
    #[inline]
    fn drop(&mut self) {
        self.synchronize();
        acl!(aclrtDestroyStream(self.0.rss));
    }
}

impl AsRaw for Stream<'_> {
    type Raw = aclrtStream;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss
    }
}

impl Stream<'_> {
    #[inline]
    pub fn synchronize(&self) {
        acl!(aclrtSynchronizeStream(self.0.rss));
    }

    #[inline]
    pub fn is_complete(&self) -> bool {
        let mut status = ACL_STREAM_STATUS_RESERVED;
        acl!(aclrtStreamQuery(self.0.rss, &mut status));
        assert_ne!(status, ACL_STREAM_STATUS_RESERVED);
        status == ACL_STREAM_STATUS_COMPLETE
    }
}
