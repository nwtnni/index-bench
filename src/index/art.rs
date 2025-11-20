use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for art_sys::Rowex<u64> {
    const IGNORE_GET: bool = true;
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::new_u64()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ art_sys::Rowex<u64> {
    type Handle<'a>
        = art_sys::RowexRef<'a, u64>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl index::IndexPin<u64> for art_sys::RowexRef<'_, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        art_sys::RowexRef::get_u64(self, key);
        None
    }

    fn insert(&mut self, key: u64, _: u64) -> Option<u64> {
        art_sys::RowexRef::insert_u64(self, key);
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        self.insert(key, value);
        None
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        art_sys::RowexRef::remove_u64(self, key);
        None
    }
}
