use std::rc::Rc;

#[derive(Clone)]
pub struct ProgramState {
    inner: Rc<InnerState>,
}

impl ProgramState {
    pub fn new(db: sled::Db) -> Self {
        Self {
            inner: Rc::new(InnerState::new(db)),
        }
    }

    pub fn get_db(&self) -> sled::Db {
        self.inner.db.clone()
    }
}

struct InnerState {
    db: sled::Db,
}

impl InnerState {
    fn new(db: sled::Db) -> Self {
        Self { db }
    }
}
