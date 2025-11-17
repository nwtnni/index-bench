use crate::Index;
use crate::index;

impl<K: index::Key, H: index::Hasher> Index<K, H> for art_sys::Rowex {
    const IGNORE_GET: bool = true;
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

impl<K: index::Key, H: index::Hasher> index::IndexSend<K, H> for &'_ art_sys::Rowex {
    type Handle<'a>
        = art_sys::RowexRef<'a>
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        art_sys::Rowex::pin(self)
    }
}

impl<K: index::Key> index::IndexPin<K> for art_sys::RowexRef<'_> {
    fn get(&mut self, key: &K) -> Option<u32> {
        art_sys::RowexRef::get(self, unsafe { core::mem::transmute_copy::<K, u64>(key) });
        None
    }

    fn insert(&mut self, key: K, _: u32) -> Option<u32> {
        art_sys::RowexRef::insert(self, unsafe { core::mem::transmute_copy::<K, u64>(&key) });
        None
    }

    fn update(&mut self, key: K, value: u32) -> Option<u32> {
        self.insert(key, value);
        None
    }

    fn remove(&mut self, key: K) -> Option<u32> {
        art_sys::RowexRef::remove(self, unsafe { core::mem::transmute_copy::<K, u64>(&key) });
        None
    }
}
