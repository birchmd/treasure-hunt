use {
    self::{
        current_clue::CurrentClueError, leader_board::LeaderboardRow, new_session::NewSessionError,
    },
    crate::state::TeamName,
    tokio::sync::oneshot,
    treasure_hunt_core::{clues::ClueView, session::SessionId},
};

pub mod current_clue;
pub mod hint;
pub mod leader_board;
pub mod new_session;

/// Commands the app can send to the state
#[derive(Debug)]
pub enum Command {
    NewSession {
        team_name: TeamName,
        response: oneshot::Sender<Result<SessionId, NewSessionError>>,
    },
    GetCurrentClue {
        id: SessionId,
        response: oneshot::Sender<Result<Option<ClueView>, CurrentClueError>>,
    },
    HintCurrentClue {
        id: SessionId,
    },
    RevealCurrentItem {
        id: SessionId,
    },
    Leaderboard {
        response: oneshot::Sender<Vec<LeaderboardRow>>,
    },
}
