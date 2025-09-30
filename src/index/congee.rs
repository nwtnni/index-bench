use core::marker::PhantomData;

use crate::Index;
use crate::index;

pub struct Map<K: index::Key> {
    inner: congee::Congee<usize, usize>,
    _key: PhantomData<K>,
}

impl<K: index::Key, H: index::Hasher> Index<K, H> for Map<K> {
    type Handle<'a> = &'a Self;

    fn new() -> Self {
        assert!(core::any::type_name::<K>() == "u64");
        const {
            assert!(core::mem::size_of::<usize>() == 8);
        }

        Self {
            inner: congee::Congee::default(),
            _key: PhantomData,
        }
    }

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::Handle<K> for &'_ Map<K> {
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

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        let guard = &self.inner.pin();
        self.inner
            .compute_if_present(
                &unsafe { core::mem::transmute_copy::<K, usize>(&key) },
                |_| Some(value as usize),
                guard,
            )
            .map(|(old, _)| old as u32)
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        let guard = &self.inner.pin();
        self.inner
            .remove(
                &unsafe { core::mem::transmute_copy::<K, usize>(&key) },
                guard,
            )
            .map(|value| value as u32)
    }
}
