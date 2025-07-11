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
    clues: CluesGenerator,
}

impl State {
    pub fn new(config: &Config) -> io::Result<(Self, mpsc::Sender<Command>)> {
        let path = Path::new(&config.clues_path);
        let clues = Clues::from_disk(path)?;
        let iterator = Arrangements::new(clues).iterator();
        let (sender, channel) = mpsc::channel(config.state_channel_size);
        let state = Self {
            sessions: HashMap::new(),
            team_names: HashSet::new(),
            channel,
            clues: iterator,
        };
        Ok((state, sender))
    }

    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(command) = self.channel.recv().await {
                match command {
                    Command::NewSession {
                        team_name,
                        response,
                    } => command::new_session::handle(&mut self, team_name, response),
                    Command::GetCurrentClue { id, response } => {
                        command::current_clue::handle(&mut self, &id, response)
                    }
                    Command::HintCurrentClue { id } => command::hint::handle_hint(&mut self, &id),
                    Command::RevealCurrentItem { id } => {
                        command::hint::handle_reveal(&mut self, &id)
                    }
                    Command::AnswerCurrentClue {
                        id,
                        guess,
                        response,
                    } => {
                        command::answer::handle(&mut self, &id, &guess, response);
                    }
                    Command::Leaderboard { response } => {
                        command::leader_board::handle(&self, response);
                    }
                }
            }
        })
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
