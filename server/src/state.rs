use {
    crate::config::Config,
    std::{collections::HashMap, fmt, io, path::Path},
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::{
        clues::{
            Clues,
            arrangement::{Arrangements, CluesGenerator},
        },
        session::{Session, SessionId},
    },
};

pub struct State {
    sessions: HashMap<TeamName, Session>,
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
                    } => {
                        let clues = self.clues.next().expect("The iterator is never empty");
                        let session = Session::new(clues);
                        let id = session.id;
                        response.send(id).ok();
                        self.sessions.insert(team_name, session);
                    }
                }
            }
        })
    }
}

/// Commands the app can send to the state
#[derive(Debug)]
pub enum Command {
    NewSession {
        team_name: TeamName,
        response: oneshot::Sender<SessionId<4>>,
    },
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
