use tokio::sync::oneshot;

use crate::state::{State, TeamName};

#[derive(Debug)]
pub struct LeaderboardRow {
    pub team_name: TeamName,
    pub score: i32,
}

pub fn handle(state: &State, response: oneshot::Sender<Vec<LeaderboardRow>>) {
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
    response.send(rows).ok();
}
