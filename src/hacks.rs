use std::{cell::RefCell, collections::VecDeque, iter::Peekable, rc::Rc};

pub type Iter = Box<dyn Iterator<Item = i32>>;
pub type Bundle = Box<dyn Iterator<Item = (i32, Iter)>>;

// Explicitly forget iterator type and only retain trait object
pub fn boxit<T, I: Iterator<Item = T> + 'static>(iter: I) -> Box<dyn Iterator<Item = T>> {
    Box::new(iter)
}

pub fn flat(b: Bundle) -> Iter {
    boxit(b.flat_map(|(_, i)| i))
}

pub struct BundleReader {
    inner: Bundle,
    buffer: Iter,

    /// Items already seen (and probably reported as outputs), but not yet pulled from next()
    prepared: VecDeque<i32>,

    /// Items pulled from next() without being prepared first. This can happen with batching.
    passed_unprepared: VecDeque<i32>,
}

// Flatten, with the ability to soft_peek
impl BundleReader {
    pub fn new(bundle: Bundle) -> Self {
        BundleReader {
            inner: bundle,
            buffer: boxit(0..0),
            prepared: VecDeque::new(),
            passed_unprepared: VecDeque::new(),
        }
    }

    pub fn pop_passed_unprepared(&mut self) -> Option<i32> {
        if let Some(x) = self.passed_unprepared.pop_front() {
            return Some(x);
        }

        None
    }

    /// Prepare while not pulling more than allowed from the parent
    pub fn prepare(&mut self, pull_limit: usize) -> Option<i32> {
        let mut pull_limit = pull_limit;
        loop {
            if let Some(x) = self.passed_unprepared.pop_front() {
                return Some(x);
            }

            if let Some(x) = self.buffer.next() {
                self.prepared.push_back(x);
                return Some(x);
            }

            if pull_limit == 0 {
                return None;
            }
            if let Some((_, i)) = self.inner.next() {
                self.buffer = i;
            } else {
                return None;
            }
            pull_limit -= 1;
        }
    }
}

impl Iterator for BundleReader {
    type Item = i32;

    // TODO if layer is locked, avoid pulling too much on next()
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(x) = self.prepared.pop_front() {
                return Some(x);
            }

            if let Some(x) = self.buffer.next() {
                self.passed_unprepared.push_back(x);
                return Some(x);
            }

            if let Some((_, i)) = self.inner.next() {
                self.buffer = i;
            } else {
                return None;
            }
        }
    }
}

// Iterator with interior mutability. You can have two references to
// it and call next() on either, just not at the same time.
pub struct RcBundleReader {
    inner: Rc<RefCell<BundleReader>>,
}

impl RcBundleReader {
    pub fn new(bundle: Bundle) -> Self {
        Self {
            inner: Rc::new(RefCell::new(BundleReader::new(bundle))),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner)
        }
    }

    pub fn pop_passed_unprepared(&mut self) -> Option<i32> {
        self.inner.as_ref().borrow_mut().pop_passed_unprepared()
    }

    pub fn prepare(&mut self, pull_limit: usize) -> Option<i32> {
        self.inner.as_ref().borrow_mut().prepare(pull_limit)
    }
}

impl Iterator for RcBundleReader {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_ref().borrow_mut().next()
    }
}

pub struct Chunks {
    input: Iter,
    buf: Vec<i32>,
    num: usize,
}

impl Chunks {
    pub fn new(input: Iter, num: usize) -> Self {
        Self {
            input,
            buf: Vec::new(),
            num,
        }
    }
}

impl Iterator for Chunks {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.buf.len() >= self.num {
                let res = self.buf.clone();
                self.buf = Vec::new();
                return Some(res);
            }

            if let Some(x) = self.input.next() {
                self.buf.push(x);
            } else if self.buf.len() > 0 {
                let res = self.buf.clone();
                self.buf = Vec::new();
                return Some(res);
            } else {
                return None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc_bundle_reader() {
        let i = (1..3).map(|x| (x, boxit(10*x+1..=10*x+2)));
        let mut i = RcBundleReader::new(boxit(i));
        let mut cloned = RcBundleReader::clone(&i);
        println!("{:?}", i.next());
        println!("{:?}", i.next());
        println!("{:?}", cloned.next());
        println!("{:?}", cloned.next());
        println!("{:?}", i.next());
    }

    #[test]
    fn test_chunks() {
        let chunks = Chunks::new(boxit(0..=10), 3);
        for c in chunks {
            dbg!(c);
        }
    }
}
