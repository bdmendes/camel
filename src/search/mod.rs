use std::mem::transmute;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

pub mod perft;
pub mod picker;

pub type Depth = u8;

#[derive(Eq, PartialEq)]
#[repr(u8)]
pub enum SearchStatusValue {
    Stopped,
    Searching,
    Pondering,
}

#[derive(Default, Clone)]
pub struct SearchStatus(Arc<AtomicU8>);

impl SearchStatus {
    pub fn new(value: SearchStatusValue) -> Self {
        let status = Self::default();
        status.set(value);
        status
    }

    pub fn get(&self) -> SearchStatusValue {
        // SAFETY: We only access the wrapped val in this context.
        unsafe { transmute(self.0.load(Ordering::Acquire)) }
    }

    pub fn set(&self, value: SearchStatusValue) {
        // SAFETY: We only access the wrapped val in this context.
        let inner: u8 = unsafe { transmute(value) };
        self.0.store(inner, Ordering::Release);
    }
}
