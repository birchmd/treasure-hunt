//! Logic for (de)serializing session type.

use {
    crate::{
        clues::{
            Clue,
            status::{KnowledgeKind, Status},
        },
        session::{Session, SessionId},
    },
    serde::{Deserialize, Serialize},
    std::{
        borrow::Cow,
        time::{Duration, Instant, SystemTime},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableSession<'a> {
    id: String,
    clues: Vec<(SerializableClue<'a>, SerializableStatus)>,
    negative_points: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableClue<'a> {
    poem: Cow<'a, str>,
    hint: Cow<'a, str>,
    item: Cow<'a, str>,
    location: Cow<'a, str>,
    code: [u8; 32],
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableStatus {
    Unread,
    Seen {
        kind: SerializableKnowledgeKind,
        time: SystemTime,
    },
    Skipped {
        kind: SerializableKnowledgeKind,
        time: SystemTime,
    },
    Solved {
        kind: SerializableKnowledgeKind,
        duration: Duration,
    },
    Declined,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SerializableKnowledgeKind {
    Unaided,
    WithHint,
    KnowingItem,
}

impl<'a> From<&'a Session> for SerializableSession<'a> {
    fn from(value: &'a Session) -> Self {
        Self {
            id: value.id.to_string(),
            clues: value
                .clues
                .iter()
                .map(|(clue, status)| (clue.into(), status.into()))
                .collect(),
            negative_points: value.negative_points,
        }
    }
}

impl<'a> From<SerializableSession<'a>> for Session {
    fn from(value: SerializableSession<'a>) -> Self {
        Self {
            id: SessionId::new(&value.id).expect("Serialized session IDs must be valid"),
            clues: value
                .clues
                .into_iter()
                .map(|(clue, status)| (clue.into(), status.into()))
                .collect(),
            negative_points: value.negative_points,
        }
    }
}

impl<'a> From<&'a Clue> for SerializableClue<'a> {
    fn from(value: &'a Clue) -> Self {
        Self {
            poem: Cow::Borrowed(&value.poem),
            hint: Cow::Borrowed(&value.hint),
            item: Cow::Borrowed(&value.item),
            location: Cow::Borrowed(&value.location),
            code: value.code,
        }
    }
}

impl<'a> From<SerializableClue<'a>> for Clue {
    fn from(value: SerializableClue<'a>) -> Self {
        Self {
            poem: value.poem.into_owned(),
            hint: value.hint.into_owned(),
            item: value.item.into_owned(),
            location: value.location.into_owned(),
            code: value.code,
        }
    }
}

impl<'a> From<&'a Status> for SerializableStatus {
    fn from(value: &'a Status) -> Self {
        let now = SystemTime::now();
        let convert = |instant: &Instant| {
            let duration = instant.elapsed();
            now.checked_sub(duration)
                .expect("Clue times must be representable")
        };
        match value {
            Status::Unread => Self::Unread,
            Status::Seen { kind, time } => Self::Seen {
                kind: kind.into(),
                time: convert(time),
            },
            Status::Skipped { kind, time } => Self::Skipped {
                kind: kind.into(),
                time: convert(time),
            },
            Status::Solved { kind, duration } => Self::Solved {
                kind: kind.into(),
                duration: *duration,
            },
            Status::Declined => Self::Declined,
        }
    }
}

impl From<SerializableStatus> for Status {
    fn from(value: SerializableStatus) -> Self {
        let now = Instant::now();
        let convert = |time: SystemTime| {
            let duration = time.elapsed().expect("Clues are in the past");
            now.checked_sub(duration)
                .expect("Clue times must be representable")
        };
        match value {
            SerializableStatus::Unread => Self::Unread,
            SerializableStatus::Seen { kind, time } => Self::Seen {
                kind: kind.into(),
                time: convert(time),
            },
            SerializableStatus::Skipped { kind, time } => Self::Skipped {
                kind: kind.into(),
                time: convert(time),
            },
            SerializableStatus::Solved { kind, duration } => Self::Solved {
                kind: kind.into(),
                duration,
            },
            SerializableStatus::Declined => Self::Declined,
        }
    }
}

impl<'a> From<&'a KnowledgeKind> for SerializableKnowledgeKind {
    fn from(value: &'a KnowledgeKind) -> Self {
        match value {
            KnowledgeKind::Unaided => Self::Unaided,
            KnowledgeKind::WithHint => Self::WithHint,
            KnowledgeKind::KnowingItem => Self::KnowingItem,
        }
    }
}

impl From<SerializableKnowledgeKind> for KnowledgeKind {
    fn from(value: SerializableKnowledgeKind) -> Self {
        match value {
            SerializableKnowledgeKind::Unaided => Self::Unaided,
            SerializableKnowledgeKind::WithHint => Self::WithHint,
            SerializableKnowledgeKind::KnowingItem => Self::KnowingItem,
        }
    }
}

#[test]
fn test_serialization_round_trip() {
    let clues = crate::clues::Clues::mock();
    let session = Session::new(clues);
    let json = session.to_json().unwrap();
    std::thread::sleep(Duration::from_millis(50));
    let round_trip = Session::from_json(json).unwrap();
    assert_eq!(session, round_trip);
}
