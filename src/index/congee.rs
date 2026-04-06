use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, u64, H> for congee::Congee<usize, usize> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        congee::Congee::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, u64, H> for &'_ congee::Congee<usize, usize> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64, u64> for &'_ congee::Congee<usize, usize> {
    fn get(&mut self, key: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::get(self, &(key as usize), &guard).map(|value| value as u64)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::insert(self, key as usize, value as usize, &guard)
            .unwrap()
            .map(|value| value as u64)
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::compute_if_present(self, &(key as usize), |_| Some(value as usize), &guard)
            .map(|(old, _)| old as u64)
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::remove(self, &(key as usize), &guard).map(|value| value as u64)
    }

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        const {
            assert!(core::mem::align_of::<(usize, usize)>() == core::mem::align_of::<u64>());
        }

        // HACK: work around congee API
        // Reserve enough room for (usize, usize)
        buffer.resize(count * 2, 0);
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast::<(usize, usize)>(), count)
        };

        let guard = self.pin();
        congee::Congee::range(self, &(key as usize), &usize::MAX, buffer, &guard);
    }
}
