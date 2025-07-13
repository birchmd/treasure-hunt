use {
    crate::state::{State, TeamName},
    tokio::sync::oneshot,
};

#[derive(Debug)]
pub struct LeaderboardRow {
    pub team_name: TeamName,
    pub score: i32,
}

pub fn handle(
    state: &State,
    maybe_id: String,
    response: oneshot::Sender<(Vec<LeaderboardRow>, Option<TeamName>)>,
) {
    let team_name = state.get_team_name(&maybe_id).cloned();
    let mut rows = Vec::new();
    for team_session in state.sessions.values() {
        let score = team_session.session.total_score();
        let row = LeaderboardRow {
            team_name: team_session.name.clone(),
            score,
        };
        rows.push(row);
    }
    rows.sort_by_key(|r| -r.score);
    response.send((rows, team_name)).ok();
}
