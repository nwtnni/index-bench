pub trait Index {
    type Handle: Handle;
    fn pin(&self) -> Self::Handle;
}

pub trait Handle {
    fn get(&mut self, key: u64) -> Option<u32>;

    fn insert(&mut self, key: u64, value: u32) -> Option<u32>;
}
