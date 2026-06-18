#![allow(unused)]
struct RingBuffer<const N: usize> {
    buf: [u8; N],
    count: usize,
    /// Location of next store
    pos: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        RingBuffer {
            buf: [0; N],
            count: 0,
            pos: 0,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn insert(&mut self, value: u8) {
        unsafe { *self.buf.get_unchecked_mut(self.pos) = value };
        self.pos = (self.pos + 1) % N;
        self.count = (self.count + 1).min(N);
    }
    pub fn count(&self) -> usize {
        self.count
    }
    /// Returns the underlying slice as is. It is not necessarily in insertion
    /// order.
    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.count]
    }
}

