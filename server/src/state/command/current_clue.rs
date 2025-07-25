use {
    crate::state::{
        State, TeamName,
        command::{ClueOrScore, Either},
    },
    std::fmt,
    tokio::sync::oneshot,
    treasure_hunt_core::{clues::ClueView, session::SessionId},
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

pub async fn handle(
    state: &mut State,
    id: &SessionId,
    response: oneshot::Sender<Result<(TeamName, ClueOrScore), CurrentClueError>>,
) {
    response.send(inner(state, id)).ok();
    state.writer.send(state.serialize()).await.ok();
}

fn inner(
    state: &mut State,
    id: &SessionId,
) -> Result<(TeamName, Either<ClueView, i32>), CurrentClueError> {
    let team_session = state
        .sessions
        .get_mut(id)
        .ok_or(CurrentClueError::UnknownSessionId)?;
    let result = team_session.session.current_clue().map_or_else(
        || Either::Right(team_session.session.total_score()),
        Either::Left,
    );
    Ok((team_session.name.clone(), result))
}
