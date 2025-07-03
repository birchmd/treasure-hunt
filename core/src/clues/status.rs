use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Unread,
    Seen {
        kind: KnowledgeKind,
        time: Instant,
    },
    Skipped {
        kind: KnowledgeKind,
        time: Instant,
    },
    Solved {
        kind: KnowledgeKind,
        duration: Duration,
    },
    Declined,
}

/// A separate struct to represent the status of the current clue.
/// The current clue is either Seen or Skipped (if the team has come back
/// to a previously skipped clue), so the `inner` field here cannot be
/// any old variant of `Status`. The `inner` field is private to uphold this
/// invariant.
#[derive(Debug)]
pub struct CurrentClueStatus<'a> {
    inner: &'a mut Status,
}

impl<'a> CurrentClueStatus<'a> {
    pub fn new(status: &'a mut Status) -> Option<Self> {
        match status {
            Status::Seen { .. } | Status::Skipped { .. } => Some(Self { inner: status }),
            Status::Unread | Status::Solved { .. } | Status::Declined => None,
        }
    }

    pub fn solved(mut self) -> &'a mut Status {
        let (kind, time) = self.unpack();
        let duration = time.elapsed();
        *self.inner = Status::Solved { kind, duration };
        self.inner
    }

    pub fn skip(self) {
        match self.inner {
            Status::Seen { kind, time } => {
                *self.inner = Status::Skipped {
                    kind: *kind,
                    time: *time,
                };
            }
            Status::Skipped { .. } => {
                // Skipping a clue a second time means you are giving up on it forever.
                *self.inner = Status::Declined;
            }
            _ => unreachable!(),
        }
    }

    fn unpack(&mut self) -> (KnowledgeKind, Instant) {
        match self.inner {
            Status::Seen { kind, time } | Status::Skipped { kind, time } => (*kind, *time),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum KnowledgeKind {
    Unaided,
    WithHint,
    KnowingItem,
}

impl KnowledgeKind {
    pub fn score(&self) -> i32 {
        match self {
            KnowledgeKind::Unaided => 300,
            KnowledgeKind::WithHint => 200,
            KnowledgeKind::KnowingItem => 100,
        }
    }
}

impl Status {
    /// Scoring:
    /// - 300 points for solving unaided
    /// - 200 points for solving with hint
    /// - 100 points for finding the item after it is revealed to you
    /// - up to 100 bonus points depending on how quickly you found the answer
    pub fn score(&self) -> i32 {
        match self {
            Self::Solved { kind, duration } => kind.score() + duration_bonus_score(duration),
            Self::Unread | Self::Seen { .. } | Self::Skipped { .. } | Self::Declined => 0,
        }
    }
}

/// Duration bonus scoring: bonus points are awarded according to an
/// exponential decay from 100 with a half-life of 10 minutes; rounded to the nearest point.
/// I.e. bonus_points = round(100 * 2^(-t/10)), where `t` is the time in minutes.
/// We use millisecond precision for `t`.
fn duration_bonus_score(duration: &Duration) -> i32 {
    // Based on the 10 minute half life and the fact we round to the nearest point,
    // 0 points are given for times larger than 77 minutes.
    // Note 77 minutes = 4_620_000 milliseconds, which is less than i32::MAX.
    let Ok(t) = i32::try_from(duration.as_millis()) else {
        return 0;
    };
    let exponent = -f64::from(t) / 600_000.0;
    let bonus_points = (2.0_f64.powf(exponent) * 100.0).round();
    // SAFETY: By construction of the exponential decay, bonus points are bounded in `[0, 100]`,
    // which is easily represented in i32.
    unsafe { bonus_points.to_int_unchecked() }
}

#[test]
fn test_scoring() {
    let durations = [
        Duration::from_secs(0),
        Duration::from_secs(60),
        Duration::from_secs(5 * 60),
        Duration::from_secs(10 * 60),
        Duration::from_secs(15 * 60),
        Duration::from_secs(20 * 60),
        Duration::from_secs(30 * 60),
        Duration::from_secs(45 * 60),
        Duration::from_secs(77 * 60),
    ];
    let bonus_scores = [100, 93, 71, 50, 35, 25, 13, 4, 0];

    for (duration, bonus) in durations.into_iter().zip(bonus_scores) {
        assert_eq!(
            Status::Solved {
                kind: KnowledgeKind::Unaided,
                duration
            }
            .score(),
            300 + bonus
        );
        assert_eq!(
            Status::Solved {
                kind: KnowledgeKind::WithHint,
                duration
            }
            .score(),
            200 + bonus
        );
        assert_eq!(
            Status::Solved {
                kind: KnowledgeKind::KnowingItem,
                duration
            }
            .score(),
            100 + bonus
        );
        assert_eq!(Status::Declined.score(), 0);
    }
}
