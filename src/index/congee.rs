use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for congee::Congee<usize, usize> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        assert!(core::any::type_name::<K>() == "u64");
        const {
            assert!(core::mem::size_of::<usize>() == 8);
        }

        congee::Congee::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ congee::Congee<usize, usize> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ congee::Congee<usize, usize> {
    fn get(&mut self, key: &K) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::get(
            self,
            unsafe { core::mem::transmute::<&K, &usize>(key) },
            &guard,
        )
        .map(|value| value as u64)
    }

    fn insert(&mut self, key: K, value: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::insert(
            self,
            unsafe { core::mem::transmute_copy::<K, usize>(&key) },
            value as usize,
            &guard,
        )
        .unwrap()
        .map(|value| value as u64)
    }

    fn update(&mut self, key: K, value: u64) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::compute_if_present(
            self,
            &unsafe { core::mem::transmute_copy::<K, usize>(&key) },
            |_| Some(value as usize),
            &guard,
        )
        .map(|(old, _)| old as u64)
    }

    fn remove(&mut self, key: K) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::remove(
            self,
            &unsafe { core::mem::transmute_copy::<K, usize>(&key) },
            &guard,
        )
        .map(|value| value as u64)
    }

    fn increment(&mut self, key: K) -> Option<u64> {
        let guard = self.pin();
        congee::Congee::compute_or_insert(
            self,
            unsafe { core::mem::transmute_copy::<K, usize>(&key) },
            |old| old.unwrap_or(0) + 1,
            &guard,
        )
        .unwrap()
        .map(|value| value as u64)
    }
}
