use {
    sha3::{Digest, Sha3_256},
    std::{io, path::Path},
};

pub mod arrangement;
mod on_disk;
pub mod status;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clue {
    pub poem: String,
    pub hint: String,
    pub item: String,
    pub location: String,
    pub code: [u8; 32],
}

impl Clue {
    #[cfg(test)]
    pub fn mock(seed: u64, location: &'static str) -> Self {
        let code = answer_to_code(&seed.to_string());
        let poem = hex::encode(code);
        let hint = poem.clone();
        let item = poem.clone();
        Self {
            poem,
            hint,
            item,
            location: location.into(),
            code,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clues(pub Vec<Clue>);

impl Clues {
    pub fn from_disk(path: &Path) -> Result<Self, io::Error> {
        let clues = on_disk::Clues::read_json(path)?
            .0
            .into_iter()
            .map(|clue| Clue {
                poem: clue.poem,
                hint: clue.hint,
                item: clue.item,
                location: clue.location,
                code: answer_to_code(&clue.answer),
            });
        Ok(Self(clues.collect()))
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self(vec![
            Clue::mock(0, "A"),
            Clue::mock(1, "B"),
            Clue::mock(2, "C"),
            Clue::mock(3, "D"),
            Clue::mock(4, "D"),
            Clue::mock(5, "D"),
            Clue::mock(6, "E"),
            Clue::mock(7, "E"),
            Clue::mock(8, "F"),
            Clue::mock(9, "F"),
            Clue::mock(10, "G"),
            Clue::mock(11, "G"),
            Clue::mock(12, "H"),
            Clue::mock(13, "H"),
        ])
    }
}

pub fn answer_to_code(answer: &str) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(answer.as_bytes());
    hasher.finalize().into()
}
