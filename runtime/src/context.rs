use crate::{
    bindings::{aclrtContext, aclrtGetCurrentContext},
    Device,
};
use context_spore::{AsRaw, RawContainer};
use std::{
    mem::{align_of, size_of},
    ptr::null_mut,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Context {
    ctx: aclrtContext,
    dev: u32,
    primary: bool,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        const { assert!(size_of::<Context>() == size_of::<[usize; 2]>()) }
        const { assert!(align_of::<Context>() == align_of::<usize>()) }

        let current = get_current_ctx();

        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        acl!(aclrtCreateContext(&mut ctx, dev as _));

        if let Some(current) = current {
            acl!(aclrtSetCurrentContext(current));
        }

        Context {
            ctx,
            dev,
            primary: false,
        }
    }

    #[inline]
    pub fn fetch_default(&self) -> Context {
        let current = get_current_ctx();

        let dev = unsafe { self.as_raw() };
        let mut ctx = null_mut();
        acl!(aclrtSetDevice(dev as _));
        acl!(aclrtGetCurrentContext(&mut ctx));

        if let Some(current) = current {
            acl!(aclrtSetCurrentContext(current));
        }

        Context {
            ctx,
            dev,
            primary: true,
        }
    }
}

impl Drop for Context {
    #[inline]
    fn drop(&mut self) {
        if !self.primary {
            acl!(aclrtDestroyContext(self.ctx));
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl AsRaw for Context {
    type Raw = aclrtContext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.ctx
    }
}

impl Context {
    #[inline]
    pub fn device(&self) -> Device {
        Device::new(self.dev)
    }

    #[inline]
    pub fn apply<T>(&self, f: impl FnOnce(&CurrentCtx) -> T) -> T {
        // 先检查当前上下文
        match get_current_ctx() {
            Some(current) => {
                if current == self.ctx {
                    // 当前上下文是目标上下文
                    // 直接执行
                    f(&CurrentCtx(self.ctx))
                } else {
                    // 当前上下文不是目标上下文
                    // 加载目标上下文
                    acl!(aclrtSetCurrentContext(self.ctx));
                    // 执行依赖上下文的操作
                    let ans = f(&CurrentCtx(self.ctx));
                    // 原上下文非空，还原上下文
                    acl!(aclrtSetCurrentContext(current));
                    ans
                }
            }
            None => {
                // 当前上下文不是目标上下文
                // 加载目标上下文
                acl!(aclrtSetCurrentContext(self.ctx));
                // 执行依赖上下文的操作
                f(&CurrentCtx(self.ctx))
            }
        }
    }
}

#[repr(transparent)]
pub struct CurrentCtx(aclrtContext);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NoCtxError;

impl AsRaw for CurrentCtx {
    type Raw = aclrtContext;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl CurrentCtx {
    #[inline]
    pub fn dev(&self) -> Device {
        let mut dev = 0;
        acl!(aclrtGetDevice(&mut dev));
        Device::new(dev as _)
    }

    /// 同步上下文对应的卡上所有上下文的所有流。
    #[inline]
    pub fn sync_device(&self) {
        acl!(aclrtSynchronizeDevice());
    }

    /// 如果存在当前上下文，在当前上下文上执行依赖上下文的操作。
    #[inline]
    pub fn apply_current<T>(f: impl FnOnce(&Self) -> T) -> Result<T, NoCtxError> {
        get_current_ctx()
            .ok_or(NoCtxError)
            .map(|current| f(&Self(current)))
    }

    /// 直接指定当前上下文，并执行依赖上下文的操作。
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context.
    #[inline]
    pub unsafe fn apply_current_unchecked<T>(raw: aclrtContext, f: impl FnOnce(&Self) -> T) -> T {
        f(&Self(raw))
    }

    /// Designates `raw` as the current context.
    ///
    /// # Safety
    ///
    /// The `raw` context must be the current pushed context.
    /// Generally, this method only used for [`RawContainer::ctx`] with limited lifetime.
    #[inline]
    pub unsafe fn from_raw<'ctx>(raw: &aclrtContext) -> &'ctx Self {
        &*(raw as *const _ as *const _)
    }

    /// Wrap a raw object in a `RawContainer`.
    ///
    /// # Safety
    ///
    /// The raw object must be created in this [`Context`].
    #[inline]
    pub unsafe fn wrap_raw<T: Unpin>(&self, rss: T) -> RawContainer<aclrtContext, T> {
        RawContainer { ctx: self.0, rss }
    }
}

fn get_current_ctx() -> Option<aclrtContext> {
    let mut current = null_mut();
    match unsafe { aclrtGetCurrentContext(&mut current) } {
        0 => Some(current),
        107002 => None,
        err => panic!("aclrtGetCurrentContext failed with error code {err}"),
    }
}

#[test]
fn test_behavior() {
    use crate::{bindings::aclrtSetCurrentContext, Device};
    use std::ptr::null_mut;

    crate::init();
    if Device::count() == 0 {
        return;
    }
    let mut default = null_mut();
    let mut context = null_mut();
    // set device 将当前上下文设置为默认上下文
    acl!(aclrtSetDevice(0));
    acl!(aclrtGetCurrentContext(&mut default));
    assert!(!default.is_null());
    // create context 将当前上下文设置为新创建的上下文
    acl!(aclrtCreateContext(&mut context, 0));
    assert!(!context.is_null());
    // set current context 设置当前上下文
    let mut current = null_mut();
    acl!(aclrtSetCurrentContext(default));
    acl!(aclrtGetCurrentContext(&mut current));
    assert_eq!(current, default);
    // 无法卸载上下文
    assert_ne!(unsafe { aclrtSetCurrentContext(null_mut()) }, 0);
    acl!(aclrtGetCurrentContext(&mut current));
    assert_eq!(current, default);
    // set device 将当前上下文设置为默认上下文
    acl!(aclrtSetDevice(0));
    acl!(aclrtGetCurrentContext(&mut current));
    assert_eq!(current, default);

    crate::finalize()
}
