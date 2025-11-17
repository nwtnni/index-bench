use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for contrie::CloneConMap<K, u32> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        contrie::CloneConMap::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ contrie::CloneConMap<K, u32> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<K: index::Key> index::IndexPin<K> for &'_ contrie::CloneConMap<K, u32> {
    fn get(&mut self, key: &K) -> Option<u32> {
        contrie::CloneConMap::get(self, key).map(|(_, value)| value)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        contrie::CloneConMap::insert(self, key, value).map(|(_, value)| value)
    }
}
