use {crate::state::State, treasure_hunt_core::session::SessionId};

pub async fn handle_hint(state: &mut State, id: &SessionId) {
    let Some(team_session) = state.sessions.get_mut(id) else {
        return;
    };
    team_session.session.hint_current_clue();
    state.writer.send(state.serialize()).await.ok();
}

pub async fn handle_reveal(state: &mut State, id: &SessionId) {
    let Some(team_session) = state.sessions.get_mut(id) else {
        return;
    };
    team_session.session.reveal_current_item();
    state.writer.send(state.serialize()).await.ok();
}

pub async fn handle_skip(state: &mut State, id: &SessionId) {
    let Some(team_session) = state.sessions.get_mut(id) else {
        return;
    };
    team_session.session.skip_current_clue();
    state.writer.send(state.serialize()).await.ok();
}
