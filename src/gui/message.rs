/// Message to be used for widgets stored in an ordered collection like `Vec`
/// or in any form that needs to distinguish one from the other when multiple
/// of the same widgets are used.
#[derive(Debug, Clone)]
pub struct IndexedMessage<I, T> {
    index: I,
    message: T,
}

impl<I, T> IndexedMessage<I, T> {
    pub fn new(index: I, message: T) -> Self {
        Self { index, message }
    }

    pub fn index(&self) -> I
    where
        I: Copy,
    {
        self.index
    }

    pub fn message(self) -> T {
        self.message
    }
}
