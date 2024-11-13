use crate::{bindings::aclrtEvent, CurrentCtx, Stream};
use context_spore::{impl_spore, AsRaw};
use std::{marker::PhantomData, ptr::null_mut, time::Duration};

impl_spore!(Event and EventSpore by (CurrentCtx, aclrtEvent));

impl<'ctx> Stream<'ctx> {
    pub fn record(&self) -> Event<'ctx> {
        let mut event = null_mut();
        acl!(aclrtCreateEvent(&mut event));
        acl!(aclrtRecordEvent(event, self.as_raw()));
        Event(unsafe { self.ctx().wrap_raw(event) }, PhantomData)
    }
}

impl Drop for Event<'_> {
    fn drop(&mut self) {
        acl!(aclrtDestroyEvent(self.0.rss))
    }
}

impl AsRaw for Event<'_> {
    type Raw = aclrtEvent;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.rss
    }
}

impl Stream<'_> {
    #[inline]
    pub fn wait_for(&self, event: &Event) {
        acl!(aclrtStreamWaitEvent(self.as_raw(), event.0.rss))
    }

    pub fn bench(&self, mut f: impl FnMut(usize, &Self), times: usize, warm_up: usize) -> Duration {
        for i in 0..warm_up {
            f(i, self);
        }
        let start = self.record();
        for i in 0..times {
            f(i, self);
        }
        let end = self.record();
        end.synchronize();
        end.elapse_from(&start).div_f32(times as _)
    }
}

impl Event<'_> {
    #[inline]
    pub fn synchronize(&self) {
        acl!(aclrtSynchronizeEvent(self.0.rss))
    }

    #[inline]
    pub fn elapse_from(&self, start: &Self) -> Duration {
        let mut ms = 0.0;
        acl!(aclrtEventElapsedTime(&mut ms, start.0.rss, self.0.rss));
        Duration::from_secs_f32(ms * 1e-3)
    }
}
