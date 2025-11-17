use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for crossbeam_skiplist::SkipMap<K, u32> {
    type Send<'a> = &'a Self;

    const IGNORE_INSERT: bool = true;
    const IGNORE_GET: bool = true;

    fn new(_: &index::Config) -> Self {
        crossbeam_skiplist::SkipMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H>
    for &crossbeam_skiplist::SkipMap<K, u32>
{
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ crossbeam_skiplist::SkipMap<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        Some(*crossbeam_skiplist::SkipMap::get(self, key)?.value())
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key, value).value())
    }
}
