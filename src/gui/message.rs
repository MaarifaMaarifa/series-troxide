/// Message to be used for widgets stored in an ordered collection like `Vec`
/// or in any form that needs to distinguish one from the other when multiple
/// of the same widgets are used.
#[derive(Debug, Clone)]
pub struct IndexedMessage<I, M> {
    index: I,
    message: M,
}

impl<I, M> IndexedMessage<I, M> {
    pub fn new(index: I, message: M) -> Self {
        Self { index, message }
    }

    pub fn index(&self) -> I
    where
        I: Copy,
    {
        self.index
    }

    pub fn message(self) -> M {
        self.message
    }
}
