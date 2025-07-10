use {
    crate::state::State,
    std::fmt,
    tokio::sync::oneshot,
    treasure_hunt_core::{
        clues::{Clue, status::KnowledgeKind},
        session::SessionId,
    },
};

#[derive(Debug)]
pub enum CurrentClueError {
    UnknownSessionId,
}

impl fmt::Display for CurrentClueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Unknown session ID")
    }
}

impl std::error::Error for CurrentClueError {}

pub fn handle(
    state: &mut State,
    id: &SessionId,
    response: oneshot::Sender<Result<Option<(Clue, KnowledgeKind)>, CurrentClueError>>,
) {
    response.send(inner(state, id)).ok();
}

fn inner(
    state: &mut State,
    id: &SessionId,
) -> Result<Option<(Clue, KnowledgeKind)>, CurrentClueError> {
    let team_session = state
        .sessions
        .get_mut(id)
        .ok_or(CurrentClueError::UnknownSessionId)?;
    Ok(team_session.session.current_clue())
}
