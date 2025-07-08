use {
    crate::state::{State, TeamName, TeamSession},
    std::fmt,
    tokio::sync::oneshot,
    treasure_hunt_core::session::{Session, SessionId},
};

#[derive(Debug)]
pub enum NewSessionError {
    DuplicateTeamName,
}

impl fmt::Display for NewSessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Duplicate team name")
    }
}

impl std::error::Error for NewSessionError {}

pub fn handle(
    state: &mut State,
    team_name: TeamName,
    response: oneshot::Sender<Result<SessionId, NewSessionError>>,
) {
    if state.team_names.contains(&team_name) {
        response.send(Err(NewSessionError::DuplicateTeamName)).ok();
        return;
    }
    let clues = state.clues.next().expect("The iterator is never empty");
    let session = Session::new(clues);
    let id = session.id;
    response.send(Ok(id)).ok();
    state.team_names.insert(team_name.clone());
    state
        .sessions
        .insert(id, TeamSession::new(team_name, session));
}
