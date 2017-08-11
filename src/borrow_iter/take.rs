
pub struct BTake<'a, I: Iterator + 'a> {
    iter: &'a mut I,
    take: usize,
    count: usize,
}

impl<'a, I: Iterator> Iterator for BTake<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.take {
            self.count += 1;
            return self.iter.next();
        }
        else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.take, None)
    }
}

pub trait BorrowTake<T: Iterator> {

    fn borrow_take<'a>(&'a mut self, take: usize) -> BTake<'a, T>;
}

impl<T> BorrowTake<T> for T where T: Iterator {

    fn borrow_take<'a>(&'a mut self, take: usize) -> BTake<'a, T> {
        BTake { iter: self, take, count: 0 }
    }
}
