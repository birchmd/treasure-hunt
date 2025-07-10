use {crate::state::State, treasure_hunt_core::session::SessionId};

pub fn handle(state: &mut State, id: &SessionId) {
    let Some(team_session) = state.sessions.get_mut(id) else {
        return;
    };
    team_session.session.hint_current_clue();
}
