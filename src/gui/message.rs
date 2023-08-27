/// Usefull message to be used for items stored in a collection like `Vec`
#[derive(Debug, Clone)]
pub struct IndexedMessage<T> {
    index: usize,
    message: T,
}

impl<T> IndexedMessage<T> {
    /// Create `IndexedMessage`
    pub fn new(index: usize, message: T) -> Self {
        Self { index, message }
    }

    /// Get the index of the element to which the message belong
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn message(self) -> T {
        self.message
    }
}
