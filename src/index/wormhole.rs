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
        wormhole_sys::WormRef::get(self, unsafe { core::mem::transmute_copy::<K, u64>(key) })
    }

    fn insert(&mut self, key: K, value: u32) -> Option<u32> {
        wormhole_sys::WormRef::put(
            self,
            unsafe { core::mem::transmute_copy::<K, u64>(&key) },
            value,
        );
        None
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        self.insert(key, value);
        None
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        wormhole_sys::WormRef::del(self, unsafe { core::mem::transmute_copy::<K, u64>(&key) });
        None
    }
}
