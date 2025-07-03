use {
    crate::clues::{
        self, Clue, Clues,
        status::{CurrentClueStatus, KnowledgeKind, Status},
    },
    rand::Rng,
    std::time::Instant,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId<const N: usize>(pub [u8; N]);

impl<const N: usize> SessionId<N> {
    pub fn new(code: &str) -> Option<Self> {
        if !Self::validate_code(code) {
            return None;
        }

        Some(Self(code.as_bytes().try_into().unwrap()))
    }

    pub fn random() -> Self {
        let mut buf = [0u8; N];
        let mut rng = rand::rng();
        for x in buf.iter_mut() {
            *x = rng.random_range(b'A'..=b'Z');
        }
        Self(buf)
    }

    fn validate_code(code: &str) -> bool {
        code.is_ascii() && code.len() == N
    }
}

pub struct Session {
    pub id: SessionId<4>,
    pub start_time: Instant,
    pub clues: Vec<(Clue, Status)>,
    pub negative_point: i32,
}

impl Session {
    pub fn new(clues: Clues) -> Self {
        Self {
            id: SessionId::random(),
            start_time: Instant::now(),
            clues: clues
                .0
                .into_iter()
                .map(|clue| (clue, Status::Unread))
                .collect(),
            negative_point: 0,
        }
    }

    pub fn current_clue(&mut self) -> Option<Clue> {
        let (clue, _) = self.inner_current_clue()?;
        Some(clue.clone())
    }

    pub fn try_solve(&mut self, submitted_answer: &str) -> Option<i32> {
        let submitted_code = clues::answer_to_code(submitted_answer);
        let (clue, status) = self.inner_current_clue()?;

        if clue.code == submitted_code {
            // They got it right!
            let status = status.solved();
            return Some(status.score());
        }

        // The answer is not right, check if it matches some other clue
        let matches_other_clue = self
            .clues
            .iter()
            .any(|(clue, _)| clue.code == submitted_code);
        if matches_other_clue {
            // -100 points for submitting the answer for a different clue.
            self.negative_point = self.negative_point.saturating_sub(100);
            return Some(-100);
        }

        None
    }

    pub fn skip_current_clue(&mut self) {
        let Some((_, status)) = self.inner_current_clue() else {
            return;
        };
        status.skip();
    }

    fn inner_current_clue<'a>(&'a mut self) -> Option<(&'a mut Clue, CurrentClueStatus<'a>)> {
        let mut first_skipped_clue: Option<(&'a mut Clue, CurrentClueStatus<'a>)> = None;

        // First pass: look for current clue (Seen) or next new clue (Unread).
        for (clue, status) in self.clues.iter_mut() {
            match status {
                Status::Seen { .. } => {
                    return Some((clue, CurrentClueStatus::new(status).unwrap()));
                }
                Status::Unread => {
                    // Set clue as being seen
                    *status = Status::Seen {
                        kind: KnowledgeKind::Unaided,
                        time: Instant::now(),
                    };
                    return Some((clue, CurrentClueStatus::new(status).unwrap()));
                }
                Status::Skipped { .. } if first_skipped_clue.is_none() => {
                    first_skipped_clue = Some((clue, CurrentClueStatus::new(status).unwrap()));
                }
                _ => continue,
            }
        }

        // If there are no more unread clues and no clue currently marked as `Seen`
        // then we are on to the `Skipped` clues.
        first_skipped_clue
    }
}
