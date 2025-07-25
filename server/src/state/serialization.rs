use {
    crate::state::{State, TeamName, TeamSession},
    serde::{Deserialize, Serialize},
    std::{
        borrow::Cow,
        collections::{HashMap, HashSet},
    },
    treasure_hunt_core::session::{Session, SessionId},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableState<'a> {
    sessions: HashMap<String, SerializableTeamSession<'a>>,
    team_names: HashSet<Cow<'a, str>>,
}

impl<'a> SerializableState<'a> {
    pub fn convert(self) -> Option<(HashMap<SessionId, TeamSession>, HashSet<TeamName>)> {
        let sessions: Option<HashMap<SessionId, TeamSession>> = self
            .sessions
            .into_iter()
            .map(|(id, session)| Some((SessionId::new(&id)?, session.try_into().ok()?)))
            .collect();
        let team_names: anyhow::Result<HashSet<TeamName>> = self
            .team_names
            .iter()
            .map(|name| TeamName::new(name))
            .collect();
        Some((sessions?, team_names.ok()?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableTeamSession<'a> {
    name: Cow<'a, str>,
    session: serde_json::Value,
}

impl<'a> TryFrom<&'a State> for SerializableState<'a> {
    type Error = serde_json::Error;

    fn try_from(value: &'a State) -> Result<Self, Self::Error> {
        let sessions: Result<HashMap<String, SerializableTeamSession<'a>>, Self::Error> = value
            .sessions
            .iter()
            .map(|(id, session)| Ok((id.to_string(), session.try_into()?)))
            .collect();
        Ok(Self {
            sessions: sessions?,
            team_names: value
                .team_names
                .iter()
                .map(|name| Cow::Borrowed(name.0.as_str()))
                .collect(),
        })
    }
}

impl<'a> TryFrom<&'a TeamSession> for SerializableTeamSession<'a> {
    type Error = serde_json::Error;

    fn try_from(value: &'a TeamSession) -> Result<Self, Self::Error> {
        Ok(Self {
            name: Cow::Borrowed(&value.name.0),
            session: value.session.to_json()?,
        })
    }
}

impl<'a> TryFrom<SerializableTeamSession<'a>> for TeamSession {
    type Error = anyhow::Error;

    fn try_from(value: SerializableTeamSession<'a>) -> Result<Self, Self::Error> {
        let name = TeamName::new(&value.name)?;
        let session = Session::from_json(value.session)?;
        Ok(Self { name, session })
    }
}
