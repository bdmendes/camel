use std::mem;
use std::sync::Arc;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::Ordering;

#[derive(Eq, PartialEq, Debug)]
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
        unsafe { mem::transmute(self.0.load(Ordering::Acquire)) }
    }

    pub fn set(&self, value: SearchStatusValue) {
        self.0.store(value as u8, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_default() {
        let status = SearchStatus::default();
        assert_eq!(status.get(), SearchStatusValue::Stopped);
    }

    #[test]
    fn status_get_set() {
        let status = SearchStatus::new(SearchStatusValue::Searching);
        assert_eq!(status.get(), SearchStatusValue::Searching);
        status.set(SearchStatusValue::Stopped);
        assert_eq!(status.get(), SearchStatusValue::Stopped);
    }
}
