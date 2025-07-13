use {
    self::{
        current_clue::CurrentClueError, leader_board::LeaderboardRow, new_session::NewSessionError,
    },
    crate::state::TeamName,
    tokio::sync::oneshot,
    treasure_hunt_core::{clues::ClueView, session::SessionId},
};

pub mod answer;
pub mod current_clue;
pub mod hint;
pub mod leader_board;
pub mod new_session;

pub type ClueOrScore = Either<ClueView, i32>;

/// Commands the app can send to the state
#[derive(Debug)]
pub enum Command {
    NewSession {
        team_name: TeamName,
        response: oneshot::Sender<Result<SessionId, NewSessionError>>,
    },
    GetCurrentClue {
        id: SessionId,
        response: oneshot::Sender<Result<(TeamName, ClueOrScore), CurrentClueError>>,
    },
    HintCurrentClue {
        id: SessionId,
    },
    RevealCurrentItem {
        id: SessionId,
    },
    SkipClue {
        id: SessionId,
    },
    AnswerCurrentClue {
        id: SessionId,
        guess: String,
        response: oneshot::Sender<Result<Option<i32>, CurrentClueError>>,
    },
    Leaderboard {
        maybe_id: String,
        response: oneshot::Sender<(Vec<LeaderboardRow>, Option<TeamName>)>,
    },
}

#[derive(Debug)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
