use std::ops::{Index, IndexMut};

pub struct CircularBuffer<T, const N: usize> {
    head: usize,
    size: usize,
    data: Vec<T>,
}

impl<T, const N: usize> CircularBuffer<T, N> {
    pub fn new(size: usize) -> Self {
        Self {
            head: 0,
            size,
            data: Vec::with_capacity(size),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    pub fn push(&mut self, value: T)
    where
        T: std::fmt::Debug,
    {
        if self.data.len() < self.size {
            self.data.push(value)
        } else {
            self.data[self.head] = value;
            self.head = (self.head + 1) % self.size;
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T, N> {
        Iter {
            inner: self,
            current: self.head,
            step: 0,
        }
    }
}

impl<T, const N: usize> Index<usize> for CircularBuffer<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for CircularBuffer<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T, const N: usize> Index<std::ops::RangeTo<usize>> for CircularBuffer<T, N> {
    type Output = [T];
    fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
        &self.data[index]
    }
}

pub struct Iter<'a, T, const N: usize> {
    inner: &'a CircularBuffer<T, N>,
    current: usize,
    step: usize,
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N>
where
    T: std::fmt::Debug,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.step < self.inner.len() {
            let cur = self.current;
            let next = (cur + 1) % self.inner.len();
            self.current = next;
            self.step += 1;
            let value = self.inner.get(cur);
            value
        } else {
            None
        }
    }
}
