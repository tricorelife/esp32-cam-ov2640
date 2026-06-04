use crate::FrameInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError<T> {
    Full(T),
    ZeroCapacity(T),
}

pub struct FrameQueue<const N: usize> {
    entries: [Option<FrameInfo>; N],
    head: usize,
    len: usize,
    dropped: u64,
}

impl<const N: usize> FrameQueue<N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn push(&mut self, info: FrameInfo) -> Result<(), QueueError<FrameInfo>> {
        if N == 0 {
            return Err(QueueError::ZeroCapacity(info));
        }

        if self.is_full() {
            return Err(QueueError::Full(info));
        }

        let tail = (self.head + self.len) % N;
        self.entries[tail] = Some(info);
        self.len += 1;
        Ok(())
    }

    pub fn push_overwrite_oldest(&mut self, info: FrameInfo) {
        if N == 0 {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }

        if self.is_full() {
            self.entries[self.head] = Some(info);
            self.head = (self.head + 1) % N;
            self.dropped = self.dropped.saturating_add(1);
            return;
        }

        let _ = self.push(info);
    }

    pub fn pop(&mut self) -> Option<FrameInfo> {
        if self.len == 0 || N == 0 {
            return None;
        }

        let item = self.entries[self.head].take();
        self.head = (self.head + 1) % N;
        self.len -= 1;
        item
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }
}

impl<const N: usize> Default for FrameQueue<N> {
    fn default() -> Self {
        Self::new()
    }
}
