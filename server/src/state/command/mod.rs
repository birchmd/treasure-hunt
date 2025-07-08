use {
    self::{leader_board::LeaderboardRow, new_session::NewSessionError},
    crate::state::TeamName,
    tokio::sync::oneshot,
    treasure_hunt_core::session::SessionId,
};

pub mod leader_board;
pub mod new_session;

/// Commands the app can send to the state
#[derive(Debug)]
pub enum Command {
    NewSession {
        team_name: TeamName,
        response: oneshot::Sender<Result<SessionId, NewSessionError>>,
    },
    Leaderboard {
        response: oneshot::Sender<Vec<LeaderboardRow>>,
    },
}
