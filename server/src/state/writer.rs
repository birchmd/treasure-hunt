use {
    crate::config::Config,
    std::path::{Path, PathBuf},
    tokio::{sync::mpsc, task::JoinHandle},
};

pub struct StateWriter {
    dest_path: PathBuf,
    receiver: mpsc::Receiver<Result<String, serde_json::Error>>,
}

impl StateWriter {
    pub fn new(
        config: &Config,
        receiver: mpsc::Receiver<Result<String, serde_json::Error>>,
    ) -> Self {
        Self {
            dest_path: Path::new(&config.state_persist_path).into(),
            receiver,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(maybe_payload) = self.receiver.recv().await {
                match maybe_payload {
                    Ok(payload) => {
                        let mut output_path = self.dest_path.clone();
                        output_path.set_extension("tmp");
                        tokio::fs::write(&output_path, payload)
                            .await
                            .inspect_err(|e| tracing::error!("Failed to persist state: {e:?}"))
                            .ok();
                        tokio::fs::rename(&output_path, &self.dest_path)
                            .await
                            .inspect_err(|e| {
                                tracing::error!("Failed to update persisted state: {e:?}")
                            })
                            .ok();
                    }
                    Err(e) => tracing::error!("Failed to serialize state to persist: {e:?}"),
                }
            }
        })
    }
}
