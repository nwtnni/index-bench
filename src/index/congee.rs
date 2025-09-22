use core::marker::PhantomData;
use std::sync::Arc;

use crate::Index;
use crate::index;

pub struct Map<K: index::Key> {
    inner: Arc<congee::Congee<usize, usize>>,
    _key: PhantomData<K>,
}

impl<K: index::Key, H: index::Hasher> Index<K, H> for Map<K> {
    type Handle = Self;

    fn new() -> Self {
        assert!(core::any::type_name::<K>() == "u64");
        const {
            assert!(core::mem::size_of::<usize>() == 8);
        }

        Self {
            inner: Arc::new(congee::Congee::default()),
            _key: PhantomData,
        }
    }

    fn pin(&self) -> Self::Handle {
        Self {
            inner: Arc::clone(&self.inner),
            _key: PhantomData,
        }
    }
}

impl<K: index::Key> index::Handle<K> for Map<K> {
    fn get(&mut self, key: &K) -> Option<u32> {
        let guard = &self.inner.pin();
        self.inner
            .get(unsafe { core::mem::transmute::<&K, &usize>(key) }, guard)
            .map(|value| value as u32)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &self.inner.pin();
        self.inner
            .insert(
                unsafe { core::mem::transmute_copy::<K, usize>(&key) },
                value as usize,
                guard,
            )
            .unwrap()
            .map(|value| value as u32)
    }

    fn scan(&mut self, _key: &K, _count: usize) -> impl Iterator<Item = u32> {
        core::iter::empty()
    }
}
