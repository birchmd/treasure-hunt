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

pub async fn handle(
    state: &mut State,
    team_name: TeamName,
    response: oneshot::Sender<Result<SessionId, NewSessionError>>,
) {
    if state.team_names.contains(&team_name) {
        response.send(Err(NewSessionError::DuplicateTeamName)).ok();
        return;
    }
    let clues = state.clues.next().expect("The iterator is never empty");
    let session = {
        let mut init = Session::new(clues);
        // Prevent randomly overwriting an existing session
        // by generating new ids until we get one not in the state.
        while state.sessions.contains_key(&init.id) {
            tracing::warn!("Session ID collision: {}", init.id);
            init.id = SessionId::random();
        }
        init
    };
    let id = session.id;
    response.send(Ok(id)).ok();
    tracing::info!("Added new session. TeamName={team_name} SessionId={id}");
    state.team_names.insert(team_name.clone());
    state
        .sessions
        .insert(id, TeamSession::new(team_name, session));
    state.writer.send(state.serialize()).await.ok();
}
