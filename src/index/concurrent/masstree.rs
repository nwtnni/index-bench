use crate::Index;
use crate::index;

macro_rules! impl_index {
    ($index:ty, $map:ty) => {
        impl<H: index::Hasher> Index<$index, u64, H> for masstree::MassTree<u64> {
            type Send<'a> = &'a Self;

            fn new(_: &index::Config) -> Self {
                masstree::MassTree::default()
            }

            fn send<'a>(&'a self) -> Self::Send<'a> {
                self
            }
        }

        impl<H: index::Hasher> index::IndexSend<$index, u64, H> for &'_ masstree::MassTree<u64> {
            type Handle<'a>
                = Self
            where
                Self: 'a;

            fn pin<'a>(&'a self) -> Self::Handle<'a> {
                self
            }
        }
    };
}

impl_index!(u64, u64);

impl index::IndexPin<u64, u64> for &'_ masstree::MassTree<u64> {
    fn get(&mut self, key: u64) {
        core::hint::black_box(masstree::MassTree::get(self, &key.to_be_bytes()));
    }

    fn insert(&mut self, key: u64, value: u64) {
        core::hint::black_box(masstree::MassTree::insert(self, &key.to_be_bytes(), value));
    }

    fn remove(&mut self, key: u64) {
        let _ = core::hint::black_box(masstree::MassTree::remove(self, &key.to_be_bytes()));
    }

    fn scan(&mut self, key: u64, count: usize) {
        let guard = self.guard();
        core::hint::black_box(
            masstree::MassTree::range(
                self,
                masstree::RangeBound::Included(&key.to_be_bytes()),
                masstree::RangeBound::Unbounded,
                &guard,
            )
            .take(count)
            .count(),
        );
    }
}

impl_index!(u128, u128);

impl index::IndexPin<u128, u64> for &'_ masstree::MassTree<u64> {
    fn get(&mut self, key: u128) {
        core::hint::black_box(masstree::MassTree::get(self, &key.to_be_bytes()));
    }

    fn insert(&mut self, key: u128, value: u64) {
        core::hint::black_box(masstree::MassTree::insert(self, &key.to_be_bytes(), value));
    }

    fn remove(&mut self, key: u128) {
        let _ = core::hint::black_box(masstree::MassTree::remove(self, &key.to_be_bytes()));
    }

    fn scan(&mut self, key: u128, count: usize) {
        let guard = self.guard();
        core::hint::black_box(
            masstree::MassTree::range(
                self,
                masstree::RangeBound::Included(&key.to_be_bytes()),
                masstree::RangeBound::Unbounded,
                &guard,
            )
            .take(count)
            .count(),
        );
    }
}

impl_index!(&'static [u8], &'static [u8]);

impl index::IndexPin<&'static [u8], u64> for &'_ masstree::MassTree<u64> {
    fn get(&mut self, key: &'static [u8]) {
        core::hint::black_box(masstree::MassTree::get(self, key));
    }

    fn insert(&mut self, key: &'static [u8], value: u64) {
        core::hint::black_box(masstree::MassTree::insert(self, key, value));
    }

    fn remove(&mut self, key: &'static [u8]) {
        let _ = core::hint::black_box(masstree::MassTree::remove(self, key));
    }

    fn scan(&mut self, key: &'static [u8], count: usize) {
        let guard = self.guard();
        core::hint::black_box(
            masstree::MassTree::range(
                self,
                masstree::RangeBound::Included(key),
                masstree::RangeBound::Unbounded,
                &guard,
            )
            .take(count)
            .count(),
        );
    }
}
