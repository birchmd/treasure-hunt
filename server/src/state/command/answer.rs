use {
    crate::state::{State, command::CurrentClueError},
    tokio::sync::oneshot,
    treasure_hunt_core::session::SessionId,
};

pub fn handle(
    state: &mut State,
    id: &SessionId,
    guess: &str,
    response: oneshot::Sender<Result<Option<i32>, CurrentClueError>>,
) {
    response.send(inner(state, id, guess)).ok();
}

fn inner(state: &mut State, id: &SessionId, guess: &str) -> Result<Option<i32>, CurrentClueError> {
    let team_session = state
        .sessions
        .get_mut(id)
        .ok_or(CurrentClueError::UnknownSessionId)?;
    Ok(team_session.session.try_solve(guess))
}
