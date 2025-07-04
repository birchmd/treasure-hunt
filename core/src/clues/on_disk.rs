//! Definition of the Clues as they exist on-disk.

use {
    serde::{Deserialize, Serialize},
    std::{fs, io, path::Path},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClueDefinition {
    pub poem: String,
    pub hint: String,
    pub item: String,
    pub location: String,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clues(pub Vec<ClueDefinition>);

impl Clues {
    pub fn read_json(path: &Path) -> Result<Self, io::Error> {
        let data = fs::read_to_string(path)?;
        let clues = serde_json::from_str(&data)?;
        Ok(clues)
    }
}
