use crate::{
    bindings::aclDeviceInfo::{self, *},
    AsRaw,
};
use std::{ffi::CStr, fmt};

#[repr(transparent)]
pub struct Device(u32);

impl AsRaw for Device {
    type Raw = u32;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Device {
    #[inline]
    pub fn count() -> usize {
        let mut count = 0;
        acl!(aclrtGetDeviceCount(&mut count));
        count as _
    }

    #[inline]
    pub fn new(index: u32) -> Self {
        Self(index)
    }

    #[inline]
    pub fn fetch() -> Option<Self> {
        if Self::count() > 0 {
            Some(Self::new(0))
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static CStr {
        unsafe { CStr::from_ptr(crate::bindings::aclrtGetSocName()) }
    }

    pub fn ai_core(&self) -> usize {
        self.get(ACL_DEVICE_INFO_AI_CORE_NUM) as _
    }

    pub fn vector_core(&self) -> usize {
        self.get(ACL_DEVICE_INFO_VECTOR_CORE_NUM) as _
    }

    pub fn l2_cache(&self) -> MemSize {
        self.get(ACL_DEVICE_INFO_L2_SIZE).into()
    }

    #[inline]
    fn get(&self, device_info: aclDeviceInfo) -> i64 {
        let mut ans = 0;
        acl!(aclGetDeviceCapability(self.0, device_info, &mut ans));
        ans
    }

    #[inline]
    pub fn info(&self) -> InfoFmt {
        InfoFmt(self)
    }
}

pub struct InfoFmt<'a>(&'a Device);

impl fmt::Display for InfoFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dev{}: {}", self.0 .0, self.0.name().to_str().unwrap())?;
        writeln!(f, "  AI Core: {}", self.0.ai_core())?;
        writeln!(f, "  Vector Core: {}", self.0.vector_core())?;
        writeln!(f, "  L2 Cache: {}", self.0.l2_cache())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MemSize(pub usize);

impl fmt::Display for MemSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "0")
        } else {
            let zeros = self.0.trailing_zeros();
            if zeros >= 40 {
                write!(f, "{}TiB", self.0 >> 40)
            } else if zeros >= 30 {
                write!(f, "{}GiB", self.0 >> 30)
            } else if zeros >= 20 {
                write!(f, "{}MiB", self.0 >> 20)
            } else if zeros >= 10 {
                write!(f, "{}KiB", self.0 >> 10)
            } else {
                write!(f, "{}B", self.0)
            }
        }
    }
}

impl From<i64> for MemSize {
    #[inline]
    fn from(value: i64) -> Self {
        Self(value as _)
    }
}

impl From<usize> for MemSize {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[test]
fn test() {
    crate::init();
    for i in 0..Device::count() {
        println!("{}", Device::new(i as _).info());
    }
}
