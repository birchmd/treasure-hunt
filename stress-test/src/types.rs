use {reqwest::StatusCode, std::time::Duration};

pub struct Outcome {
    pub id: usize,
    pub elapsed: Duration,
    pub status: Option<StatusCode>,
    pub error: Option<String>,
}

impl Outcome {
    pub fn is_success(&self) -> bool {
        matches!(self.status, Some(s) if s.is_success())
    }
}
