use core::cell::RefCell;

use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, u64, H> for art_sys::Rowex<u64> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::new_u64()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, u64, H> for &'_ art_sys::Rowex<u64> {
    type Handle<'a>
        = art_sys::RowexRef<'a, u64>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl index::IndexPin<u64, u64> for art_sys::RowexRef<'_, u64> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(art_sys::RowexRef::get_u64(self, key));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(art_sys::RowexRef::insert_u64(self, key, value));
    }

    fn scan(&mut self, key: u64, count: usize) {
        thread_local! {
            static BUFFER: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
        }

        BUFFER.with_borrow_mut(|buffer| {
            buffer.resize(count, 0);
            self.get_range_u64(key, u64::MAX, buffer);
        })
    }
}

impl<H: index::Hasher> Index<u128, u64, H> for art_sys::Rowex<Vec<u8>> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::new_string()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u128, u64, H> for &'_ art_sys::Rowex<Vec<u8>> {
    type Handle<'a>
        = art_sys::RowexRef<'a, Vec<u8>>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl index::IndexPin<u128, u64> for art_sys::RowexRef<'_, Vec<u8>> {
    fn get(&mut self, key: u128) {
        let key = key.to_be_bytes();
        art_sys::RowexRef::get_string(self, &key);
    }

    fn insert(&mut self, key: u128, value: u64) {
        let key = key.to_be_bytes();
        art_sys::RowexRef::insert_string(self, &key, value);
    }

    fn update(&mut self, key: u128, value: u64) {
        let key = key.to_be_bytes();
        art_sys::RowexRef::insert_string(self, &key, value);
    }

    fn scan(&mut self, key: u128, count: usize) {
        let key = key.to_be_bytes();

        thread_local! {
            static BUFFER: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
        }

        BUFFER.with_borrow_mut(|buffer| {
            buffer.resize(count, 0);
            self.get_range_string(&key, &u128::MAX.to_be_bytes(), buffer);
        })
    }
}

impl<H: index::Hasher> Index<&'static [u8], u64, H> for art_sys::Rowex<Vec<u8>> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::new_string()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<&'static [u8], u64, H> for &'_ art_sys::Rowex<Vec<u8>> {
    type Handle<'a>
        = art_sys::RowexRef<'a, Vec<u8>>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl index::IndexPin<&'static [u8], u64> for art_sys::RowexRef<'_, Vec<u8>> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(art_sys::RowexRef::get_string(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(art_sys::RowexRef::insert_string(self, key, value));
    }

    fn update(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(art_sys::RowexRef::insert_string(self, key, value));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        thread_local! {
            static BUFFER: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
        }

        BUFFER.with_borrow_mut(|buffer| {
            buffer.resize(count, 0);

            // HACK: input data is a subset of ASCII and shouldn't contain any bytes >= 0x7F
            self.get_range_string(key, b"\x7F", buffer);
        })
    }
}
