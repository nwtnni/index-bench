use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for wormhole_sys::Wormhole {
    const IGNORE_INSERT: bool = true;

    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        assert!(core::any::type_name::<K>() == "u64");
        const {
            assert!(core::mem::size_of::<usize>() == 8);
        }
        Self::default()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ wormhole_sys::Wormhole {
    type Handle<'a>
        = wormhole_sys::WormRef<'a>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        wormhole_sys::Wormhole::pin(self)
    }
}

impl<K: index::Key> index::IndexPin<K> for wormhole_sys::WormRef<'_> {
    fn get(&mut self, key: &K) -> Option<u32> {
        key.with_ptr(|ptr| unsafe { wormhole_sys::WormRef::get(self, ptr, key.len()) })
            .map(|value| value as u32)
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        key.with_ptr(|ptr| unsafe {
            wormhole_sys::WormRef::put(self, ptr, key.len(), value as u64)
        });
        None
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        self.insert(key, value);
        None
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        key.with_ptr(|ptr| unsafe { wormhole_sys::WormRef::del(self, ptr, key.len()) });
        None
    }
}
