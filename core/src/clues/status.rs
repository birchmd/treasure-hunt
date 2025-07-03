use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Unread,
    Seen(Instant),
    Skipped(Instant),
    Solved(Duration),
    SolvedWithHint(Duration),
    SolvedKnowingItem(Duration),
    Declined,
}

impl Status {
    /// Scoring:
    /// - 300 points for solving unaided
    /// - 200 points for solving with hint
    /// - 100 points for finding the item after it is revealed to you
    /// - up to 100 bonus points depending on how quickly you found the answer
    pub fn score(&self) -> u32 {
        match self {
            Self::Solved(duration) => 300 + duration_bonus_score(duration),
            Self::SolvedWithHint(duration) => 200 + duration_bonus_score(duration),
            Self::SolvedKnowingItem(duration) => 100 + duration_bonus_score(duration),
            Self::Unread | Self::Seen(_) | Self::Skipped(_) | Self::Declined => 0,
        }
    }
}

/// Duration bonus scoring: bonus points are awarded according to an
/// exponential decay from 100 with a half-life of 10 minutes; rounded to the nearest point.
/// I.e. bonus_points = round(100 * 2^(-t/10)), where `t` is the time in minutes.
/// We use millisecond precision for `t`.
fn duration_bonus_score(duration: &Duration) -> u32 {
    // Based on the 10 minute half life and the fact we round to the nearest point,
    // 0 points are given for times larger than 77 minutes.
    // Note 77 minutes = 4_620_000 milliseconds, which is less than u32::MAX.
    let Ok(t) = u32::try_from(duration.as_millis()) else {
        return 0;
    };
    let exponent = -f64::from(t) / 600_000.0;
    let bonus_points = (2.0_f64.powf(exponent) * 100.0).round();
    // SAFETY: By construction of the exponential decay, bonus points are bounded in `[0, 100]`,
    // which is easily represented in u32.
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
        assert_eq!(Status::Solved(duration).score(), 300 + bonus);
        assert_eq!(Status::SolvedWithHint(duration).score(), 200 + bonus);
        assert_eq!(Status::SolvedKnowingItem(duration).score(), 100 + bonus);
        assert_eq!(Status::Declined.score(), 0);
    }
}
