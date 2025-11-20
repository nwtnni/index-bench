use crate::Index;
use crate::index;

impl<H: index::Hasher> Index<u64, H> for scc::HashMap<u64, u64, H> {
    type Send<'a> = &'a Self;

    fn new(_: &index::Config) -> Self {
        scc::HashMap::with_hasher(H::default())
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ scc::HashMap<u64, u64, H> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexPin<u64> for &'_ scc::HashMap<u64, u64, H> {
    fn get(&mut self, key: u64) -> Option<u64> {
        self.read_sync(&key, |_, value| *value)
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        self.upsert_sync(key, value)
    }

    fn update(&mut self, key: u64, value: u64) -> Option<u64> {
        self.update_sync(&key, |_, old| {
            let save = *old;
            *old = value;
            save
        })
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        self.remove_sync(&key).map(|(_, value)| value)
    }
}

impl<H: index::Hasher> Index<u64, H> for scc::TreeIndex<u64, u64> {
    type Send<'a> = &'a Self;

    const IGNORE_INSERT: bool = true;

    fn new(_: &index::Config) -> Self {
        scc::TreeIndex::new()
    }

    fn send<'a>(&'a self) -> Self::Send<'a> {
        self
    }
}

impl<H: index::Hasher> index::IndexSend<u64, H> for &'_ scc::TreeIndex<u64, u64> {
    type Handle<'a>
        = Self
    where
        Self: 'a;

    fn pin<'a>(&'a self) -> Self::Handle<'a> {
        *self
    }
}

impl index::IndexPin<u64> for &'_ scc::TreeIndex<u64, u64> {
    fn get(&mut self, key: u64) -> Option<u64> {
        let guard = scc::Guard::new();
        self.peek(&key, &guard).copied()
    }

    fn insert(&mut self, key: u64, value: u64) -> Option<u64> {
        // NOTE: does not insert if exists
        self.insert_sync(key, value).err().map(|(_, value)| value)
    }

    fn remove(&mut self, key: u64) -> Option<u64> {
        self.remove_sync(&key).then_some(0)
    }

    // fn range<'a>(
    //     &'a mut self,
    //     _retry_scan: usize,
    //     min: &'a K,
    //     max: &'a K,
    //     output: &mut Vec<(K, u64)>,
    // ) {
    //     let guard = scc::Guard::new();
    //     output.extend(
    //         scc::TreeIndex::range::<K, RangeInclusive<&'_ K>>(self, min..=max, &guard)
    //             .map(|(key, value)| (key.clone(), *value)),
    //     );
    // }
}
