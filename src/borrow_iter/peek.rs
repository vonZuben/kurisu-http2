
pub struct BPeekable<'a, I: Iterator + 'a> {
    iter: &'a mut I,
    peeked: Option<Option<I::Item>>,
}

impl<'a, I: Iterator> BPeekable<'a, I> {

    pub fn bpeek(&mut self) -> Option<&I::Item> {
        if self.peeked.is_none() {
            self.peeked = Some(self.iter.next());
        }
        match self.peeked {
            Some(Some(ref value)) => Some(value),
            Some(None) => None,
            _ => unreachable!(),
        }
    }
}

impl<'a, I: Iterator> Iterator for BPeekable<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(v) => v,
            None => self.iter.next(),
        }
    }
}

pub trait BorrowPeekable<T: Iterator> {

    fn borrow_peekable<'a>(&'a mut self) -> BPeekable<T>;
}

impl<T> BorrowPeekable<T> for T where T: Iterator {

    fn borrow_peekable<'a>(&'a mut self) -> BPeekable<T> {
        BPeekable { iter: self, peeked: None }
    }
}