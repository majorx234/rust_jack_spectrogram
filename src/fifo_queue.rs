pub trait FifoQueue<T> {
    fn new(size: usize) -> Self;
    fn push(&mut self, new_data: T);
    fn pop(&mut self) -> Option<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
