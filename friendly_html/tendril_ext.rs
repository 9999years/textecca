use std::borrow::{Borrow, Cow};

pub trait AsStrLossy {
    fn as_str_lossy(&self) -> Cow<str>;
}

impl<B> AsStrLossy for B
where
    B: Borrow<[u8]>,
{
    fn as_str_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.borrow())
    }
}
