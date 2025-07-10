use {
    crate::clues::{
        self, Clue, Clues,
        status::{CurrentClueStatus, KnowledgeKind, Status},
    },
    rand::Rng,
    std::{
        fmt,
        time::{Duration, Instant},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenericSessionId<const N: usize>([u8; N]);

impl<const N: usize> GenericSessionId<N> {
    pub fn new(code: &str) -> Option<Self> {
        if !Self::validate_code(code) {
            return None;
        }

        Some(Self(
            code.to_ascii_uppercase().as_bytes().try_into().unwrap(),
        ))
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

impl<const N: usize> fmt::Display for GenericSessionId<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = str::from_utf8(&self.0).map_err(|_| fmt::Error)?;
        write!(f, "{s}")
    }
}

pub type SessionId = GenericSessionId<4>;

pub struct Session {
    pub id: SessionId,
    clues: Vec<(Clue, Status)>,
    negative_points: i32,
}

impl Session {
    pub fn new(clues: Clues) -> Self {
        Self {
            id: SessionId::random(),
            clues: clues
                .0
                .into_iter()
                .map(|clue| (clue, Status::Unread))
                .collect(),
            negative_points: 0,
        }
    }

    pub fn total_score(&self) -> i32 {
        self.clues
            .iter()
            .fold(self.negative_points, |acc, (_, status)| {
                acc.saturating_add(status.score())
            })
    }

    pub fn current_clue_duration(&mut self) -> Option<Duration> {
        let (_, status) = self.inner_current_clue()?;
        Some(status.duration())
    }

    pub fn current_clue(&mut self) -> Option<(Clue, KnowledgeKind)> {
        let (clue, status) = self.inner_current_clue()?;
        Some((clue.clone(), status.get_knowledge_kind()))
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
            self.negative_points = self.negative_points.saturating_sub(100);
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

    pub fn hint_current_clue(&mut self) -> Option<String> {
        let (clue, mut status) = self.inner_current_clue()?;
        status.hinted();
        Some(clue.hint.clone())
    }

    pub fn reveal_current_item(&mut self) -> Option<String> {
        let (clue, mut status) = self.inner_current_clue()?;
        status.revealed();
        Some(clue.item.clone())
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

#[test]
fn test_session() {
    let answers: Vec<String> = (0..14).map(|x| x.to_string()).collect();
    let clues = Clues::mock();
    let mut session = Session::new(clues.clone());

    // `current_clue` is idempotent
    let clue = session.current_clue();
    assert_eq!(clue, session.current_clue());

    // We can solve the current clue
    let points = session.try_solve(&answers[0]).unwrap();
    assert!(points > 300, "We solved unaided");

    // We get a penalty for submitting an answer to a different clue
    let clue = session.current_clue();
    let points = session.try_solve(&answers[7]).unwrap();
    assert_eq!(points, -100, "Wrong submit penalty");

    // We are still on the same clue since we did not solve or skip
    assert_eq!(clue, session.current_clue());

    // Submitting a completely wrong answer does not change the state
    assert!(session.try_solve("Hello, world!").is_none());
    assert_eq!(clue, session.current_clue());

    // We can ask for a hint
    assert_eq!(session.hint_current_clue(), Some(clues.0[1].hint.clone()));

    // Solving after the hint is worth less points
    let points = session.try_solve(&answers[1]).unwrap();
    assert!(points > 200, "We solved with hint");

    // We can reveal the item
    session.current_clue();
    assert_eq!(session.reveal_current_item(), Some(clues.0[2].item.clone()));

    // Solving with item revealed is worth less points
    let points = session.try_solve(&answers[2]).unwrap();
    assert!(points > 100, "We solved knowing the item");

    // We can skip a clue
    session.current_clue();
    session.skip_current_clue();

    // And solve the next one
    session.current_clue();
    let points = session.try_solve(&answers[4]).unwrap();
    assert!(points > 300, "We solved unaided");

    // We can skip more clues
    for _ in 0..3 {
        session.current_clue();
        session.skip_current_clue();
    }

    // And solve the rest
    for a in &answers[8..] {
        session.current_clue();
        let points = session.try_solve(a).unwrap();
        assert!(points > 300, "We solved unaided");
    }

    // Now we are back to clues we skipped
    let clue = session.current_clue();
    assert_eq!(clue, Some((clues.0[3].clone(), KnowledgeKind::Unaided)));

    // If we skip a clue a second time then it is declined
    session.skip_current_clue();
    assert_eq!(session.clues[3].1, Status::Declined);

    // We can still ask for hints on skipped clues
    session.current_clue();
    assert_eq!(session.hint_current_clue(), Some(clues.0[5].hint.clone()));
    let points = session.try_solve(&answers[5]).unwrap();
    assert!(points > 200, "We solved with hint");

    session.current_clue();
    assert_eq!(session.reveal_current_item(), Some(clues.0[6].item.clone()));
    let points = session.try_solve(&answers[6]).unwrap();
    assert!(points > 100, "We solved knowing the item");

    session.current_clue();
    session.skip_current_clue();

    assert!(
        session.current_clue().is_none(),
        "After all clues are solved or declined then there is no current clue"
    );

    // + 8 Fully solved (with full bonus)
    // + 2 Solved with hint (and full bonus)
    // + 2 Solved knowing the item (and full bonus)
    // - 1 penalty for future guess
    assert_eq!(session.total_score(), 8 * 400 + 2 * 300 + 2 * 200 - 100);
}
