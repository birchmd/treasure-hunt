use {
    crate::clues::{Clue, status::Status},
    std::time::Instant,
};

pub struct Session {
    pub id: u64,
    pub start_time: Instant,
    pub clues: Vec<(Clue, Status)>,
}
