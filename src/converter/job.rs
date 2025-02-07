use uuid::Uuid;

pub struct Job {
    pub id: Uuid,
}

impl Job {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug)]
pub enum ProgressUpdate {
    Frame(u64),
    FPS(f64),
}
