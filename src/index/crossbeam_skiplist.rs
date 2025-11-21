use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for crossbeam_skiplist::SkipMap<u64, u64> {
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

impl<H: index::Hasher> index::IndexSend<u64, H> for &crossbeam_skiplist::SkipMap<u64, u64> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<u64> for &'_ crossbeam_skiplist::SkipMap<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::get(self, &key)?.value())
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key, value).value())
    }
}

impl<H: index::Hasher> Index<String, H> for crossbeam_skiplist::SkipMap<&'static str, u64> {
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

impl<H: index::Hasher> index::IndexSend<String, H>
    for &crossbeam_skiplist::SkipMap<&'static str, u64>
{
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl index::IndexPin<String> for &'_ crossbeam_skiplist::SkipMap<&'static str, u64> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::get(self, &key)?.value())
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        Some(*crossbeam_skiplist::SkipMap::insert(self, key, value).value())
    }
}
