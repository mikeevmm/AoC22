#[derive(Debug)]
pub struct CircularBuffer<const N: usize, T> {
    inner: [T; N],
    start: usize,
}

impl<const N: usize, T> From<[T; N]> for CircularBuffer<N, T> {
    fn from(inner: [T; N]) -> Self {
        CircularBuffer { inner, start: 0 }
    }
}

impl<const N: usize, T> CircularBuffer<N, T> {
    pub fn push(&mut self, value: T) {
        if self.start == 0 {
            self.start = N - 1;
        } else {
            self.start -= 1;
        }
        self.inner[self.start] = value;
    }

    pub fn oldest(&self) -> &T {
        if self.start == 0 {
            &self.inner[N - 1]
        } else {
            &self.inner[self.start - 1]
        }
    }

    /// Returns the ith element in order of newest to oldest.
    pub fn ith_newest(&self, i: usize) -> &T {
        &self.inner[(self.start + i) % N]
    }

    /// Returns the ith element in order of oldest to newest.
    pub fn ith_oldest(&self, i: usize) -> &T {
        &self.inner[(self.start + N - 1 - i) % N]
    }
}
