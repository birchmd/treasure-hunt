use {
    self::command::Command,
    crate::config::Config,
    std::{
        collections::{HashMap, HashSet},
        fmt, io,
        path::Path,
    },
    tokio::sync::mpsc,
    treasure_hunt_core::{
        clues::{
            Clues,
            arrangement::{Arrangements, CluesGenerator},
        },
        session::{Session, SessionId},
    },
};

pub mod command;
mod serialization;
pub mod writer;

pub struct TeamSession {
    pub name: TeamName,
    pub session: Session,
}

impl TeamSession {
    pub fn new(name: TeamName, session: Session) -> Self {
        Self { name, session }
    }
}

pub struct State {
    sessions: HashMap<SessionId, TeamSession>,
    team_names: HashSet<TeamName>,
    channel: mpsc::Receiver<Command>,
    writer: mpsc::Sender<Result<String, serde_json::Error>>,
    clues: CluesGenerator,
}

impl State {
    pub fn new(config: &Config) -> io::Result<(Self, mpsc::Sender<Command>, writer::StateWriter)> {
        let path = Path::new(&config.clues_path);
        let clues = Clues::from_disk(path)?;
        let iterator = Arrangements::new(clues).iterator();
        let (sender, channel) = mpsc::channel(config.state_channel_size);
        let (writer_tx, writer_rx) = mpsc::channel(config.state_channel_size);
        let state_writer = writer::StateWriter::new(config, writer_rx);
        let (sessions, team_names) = Self::load_persisted_state(config).unwrap_or_default();
        let state = Self {
            sessions,
            team_names,
            channel,
            writer: writer_tx,
            clues: iterator,
        };
        Ok((state, sender, state_writer))
    }

    pub fn serialize(&self) -> Result<String, serde_json::Error> {
        let serializable = serialization::SerializableState::try_from(self)?;
        serde_json::to_string_pretty(&serializable)
    }

    pub fn get_team_name(&self, maybe_id: &str) -> Option<&TeamName> {
        let session_id = SessionId::new(maybe_id)?;
        let session = self.sessions.get(&session_id)?;
        Some(&session.name)
    }

    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(command) = self.channel.recv().await {
                match command {
                    Command::NewSession {
                        team_name,
                        response,
                    } => command::new_session::handle(&mut self, team_name, response).await,
                    Command::GetCurrentClue { id, response } => {
                        command::current_clue::handle(&mut self, &id, response).await
                    }
                    Command::HintCurrentClue { id } => {
                        command::hint::handle_hint(&mut self, &id).await
                    }
                    Command::RevealCurrentItem { id } => {
                        command::hint::handle_reveal(&mut self, &id).await
                    }
                    Command::SkipClue { id } => command::hint::handle_skip(&mut self, &id).await,
                    Command::AnswerCurrentClue {
                        id,
                        guess,
                        response,
                    } => {
                        command::answer::handle(&mut self, &id, &guess, response).await;
                    }
                    Command::Leaderboard { maybe_id, response } => {
                        command::leader_board::handle(&self, maybe_id, response);
                    }
                }
            }
        })
    }

    fn load_persisted_state(
        config: &Config,
    ) -> Option<(HashMap<SessionId, TeamSession>, HashSet<TeamName>)> {
        let contents = std::fs::read_to_string(Path::new(&config.state_persist_path)).ok()?;
        let state: serialization::SerializableState<'static> =
            serde_json::from_str(&contents).ok()?;
        let result = state.convert();
        if let Some((_, names)) = &result {
            tracing::info!("Loaded previous state including team names: {names:?}");
        }
        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TeamName(String);

impl TeamName {
    pub fn new(name: &str) -> anyhow::Result<Self> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            anyhow::bail!("Enter a team name!");
        }
        if trimmed.len() > 50 {
            anyhow::bail!("Team name too long!");
        }
        Ok(TeamName(trimmed.into()))
    }
}

impl fmt::Display for TeamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
