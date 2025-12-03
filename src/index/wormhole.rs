use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for wormhole_sys::Wormhole {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ wormhole_sys::Wormhole {
    type Handle<'a>
        = wormhole_sys::WormRef<'a>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        wormhole_sys::Wormhole::pin(self)
    }
}

impl index::IndexPin<u64> for wormhole_sys::WormRef<'_> {
    fn get(&mut self, key: u64) -> Option<u64> {
        let key = key.to_be_bytes();
        let ptr = key.as_ptr().cast();
        unsafe { wormhole_sys::WormRef::get(self, ptr, key.len()) }
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        let key = key.to_be_bytes();
        let ptr = key.as_ptr().cast();
        unsafe { wormhole_sys::WormRef::put(self, ptr, key.len(), value) }
        None
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        <Self as index::IndexPin<u64>>::insert(self, key, value);
        None
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        let key = key.to_be_bytes();
        let ptr = key.as_ptr().cast();
        unsafe { wormhole_sys::WormRef::del(self, ptr, key.len()) };
        None
    }

    fn scan(
        &mut self,
        key: <u64 as arctic::raw::Key>::Borrow<'static>,
        count: usize,
        buffer: &mut Vec<u64>,
    ) {
        let key = key.to_be_bytes();
        let ptr = key.as_ptr().cast();
        buffer.extend(unsafe { wormhole_sys::WormRef::iter(self, ptr, key.len()) }.take(count));
    }
}

impl<H: index::Hasher> Index<Vec<u8>, H> for wormhole_sys::Wormhole {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<Vec<u8>, H> for &'_ wormhole_sys::Wormhole {
    type Handle<'a>
        = wormhole_sys::WormRef<'a>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        wormhole_sys::Wormhole::pin(self)
    }
}

impl index::IndexPin<Vec<u8>> for wormhole_sys::WormRef<'_> {
    fn get(&mut self, key: &'static [u8]) -> Option<u64> {
        unsafe { wormhole_sys::WormRef::get(self, key.as_ptr().cast(), key.len()) }
    }

    fn insert(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        unsafe { wormhole_sys::WormRef::put(self, key.as_ptr().cast(), key.len(), value) }
        None
    }

    fn update(&mut self, key: &'static [u8], value: u64) -> Option<u64> {
        <Self as index::IndexPin<Vec<u8>>>::insert(self, key, value);
        None
    }

    fn remove(&mut self, key: &'static [u8]) -> Option<u64> {
        unsafe { wormhole_sys::WormRef::del(self, key.as_ptr().cast(), key.len()) };
        None
    }

    fn scan(&mut self, key: &'static [u8], count: usize, buffer: &mut Vec<u64>) {
        buffer.extend(
            unsafe { wormhole_sys::WormRef::iter(self, key.as_ptr().cast(), key.len()) }
                .take(count),
        );
    }
}
