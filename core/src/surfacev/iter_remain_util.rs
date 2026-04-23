pub struct RemainIter<'i, V> {
    next: usize,
    input: &'i [V],
}

impl<'i, V> RemainIter<'i, V> {
    pub fn new(input: &'i [V]) -> Self {
        Self { next: 0, input }
    }
}

impl<'i, V> Iterator for RemainIter<'i, V> {
    type Item = (&'i V, &'i [V]);
    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.input.len() {
            let result = &self.input[self.next];
            self.next += 1;
            Some((result, &self.input[self.next..]))
        } else {
            None
        }
    }
}
