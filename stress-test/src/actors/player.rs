use {
    crate::types::Outcome,
    reqwest::{Client, Response},
    std::{sync::Arc, time::Instant},
    tokio::{
        sync::{Barrier, mpsc::UnboundedSender},
        task::JoinHandle,
    },
};

pub struct Player {
    channel: UnboundedSender<Outcome>,
    barrier: Arc<Barrier>,
    client: Client,
    id: usize,
    base_url: &'static str,
    answers: Vec<(String, String)>,
}

impl Player {
    pub fn spawn_swarm(
        concurrency: usize,
        base_url: &'static str,
        client: Client,
        sender: UnboundedSender<Outcome>,
        answers: Vec<(String, String)>,
    ) -> anyhow::Result<Vec<JoinHandle<anyhow::Result<()>>>> {
        // Every task blocks here until all `concurrency` tasks are ready, so the
        // requests leave as close to simultaneously as the runtime allows.
        let barrier = Arc::new(Barrier::new(concurrency));
        let mut handles = Vec::with_capacity(concurrency);
        for id in 0..concurrency {
            let player = Self {
                channel: sender.clone(),
                barrier: Arc::clone(&barrier),
                client: client.clone(),
                id,
                base_url,
                answers: answers.clone(),
            };

            handles.push(player.spawn());
        }

        Ok(handles)
    }

    fn spawn(self) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            let session_id = self.init_session().await?;
            for _ in 0..16 {
                self.solve_clue(&session_id).await?;
            }
            Ok(())
        })
    }

    async fn handle_response(&self, start: Instant, response: Response) -> anyhow::Result<String> {
        let status = response.status();
        // Read the body to completion so the timing covers the
        // whole response, not just the headers.
        let body = response.bytes().await;
        let elapsed = start.elapsed();
        let outcome = Outcome {
            id: self.id,
            elapsed,
            status: Some(status),
            error: body.as_ref().err().map(|e| e.to_string()),
        };
        self.channel.send(outcome)?;
        Ok(String::from_utf8(body?.as_ref().to_vec())?)
    }

    async fn init_session(&self) -> anyhow::Result<String> {
        self.barrier.wait().await;

        let id = self.id;
        let start = Instant::now();
        let response = self
            .client
            .post(self.base_url)
            .form(&[("team_name", format!("stress-test-{id}"))])
            .send()
            .await?;
        let body = self.handle_response(start, response).await?;
        parse_session_id(body)
    }

    async fn solve_clue(&self, session_id: &str) -> anyhow::Result<()> {
        let start = Instant::now();
        let url = format!("{}/clue/{session_id}", self.base_url);
        let response = self.client.get(url).send().await?;
        let body = self.handle_response(start, response).await?;
        let answer_url = parse_answer_url(&body)?;
        let given_poem = parse_poem(&body)?;
        let answer = self
            .answers
            .iter()
            .find_map(|(poem, answer)| {
                if given_poem.contains(poem) {
                    Some(answer)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::Error::msg("Failed to match poem"))?;
        let start = Instant::now();
        let response = self
            .client
            .post(format!("{}{answer_url}", self.base_url))
            .form(&[("clue_answer", answer)])
            .send()
            .await?;
        self.handle_response(start, response).await?;
        Ok(())
    }
}

fn parse_session_id(body: String) -> anyhow::Result<String> {
    let err = || anyhow::Error::msg("Failed to find session_id");
    let line = body
        .lines()
        .find(|l| l.contains("Your session id is"))
        .ok_or_else(err)?;
    let id = line
        .split("Your session id is")
        .nth(1)
        .ok_or_else(err)?
        .split('.')
        .next()
        .ok_or_else(err)?
        .trim();
    Ok(id.to_string())
}

fn parse_answer_url(body: &str) -> anyhow::Result<String> {
    let err = || anyhow::Error::msg("Failed to find answer url");
    let line = body
        .lines()
        .find(|l| l.contains("/answer/"))
        .ok_or_else(err)?;
    let url = line
        .split("action=")
        .nth(1)
        .ok_or_else(err)?
        .split(' ')
        .next()
        .ok_or_else(err)?
        .trim()
        .replace('"', "");
    Ok(url)
}

fn parse_poem(body: &str) -> anyhow::Result<String> {
    let err = || anyhow::Error::msg("Failed to find poem");
    let line = body
        .lines()
        .zip(body.lines().skip(1))
        .find_map(|(l1, l2)| {
            if l1.contains("<h2>Clue") {
                Some(l2)
            } else {
                None
            }
        })
        .ok_or_else(err)?;
    let poem = line
        .split("<p>")
        .nth(1)
        .ok_or_else(err)?
        .split("</p>")
        .next()
        .ok_or_else(err)?
        .trim();
    Ok(poem.to_string())
}
