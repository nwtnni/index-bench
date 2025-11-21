use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for art_sys::Rowex<u64> {
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

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        art_sys::RowexRef::insert_u64(self, key, value);
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        self.insert(key, value);
        None
    }

    fn scan(&mut self, key: u64, count: usize, buffer: &mut Vec<u64>) {
        buffer.resize(count, 0);
        self.get_range_u64(key, u64::MAX, buffer);
    }
}

impl<H: index::Hasher> Index<String, H> for art_sys::Rowex<String> {
    const IGNORE_GET: bool = true;
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::new_string()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<String, H> for &'_ art_sys::Rowex<String> {
    type Handle<'a>
        = art_sys::RowexRef<'a, String>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl index::IndexPin<String> for art_sys::RowexRef<'_, String> {
    fn get(&mut self, key: &'static str) -> Option<u64> {
        art_sys::RowexRef::get_string(self, key)
    }

    fn insert(&mut self, key: &'static str, value: u64) -> Option<u64> {
        art_sys::RowexRef::insert_string(self, key, value);
        None
    }

    fn update(&mut self, key: &'static str, value: u64) -> Option<u64> {
        self.insert(key, value);
        None
    }

    fn scan(&mut self, key: &'static str, count: usize, buffer: &mut Vec<u64>) {
        buffer.resize(count, 0);
        // HACK: input data is a subset of ASCII and shouldn't contain any bytes >= 0x7F
        self.get_range_string(key, "\x7F", buffer);
    }
}
