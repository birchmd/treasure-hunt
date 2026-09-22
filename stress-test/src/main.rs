use {reqwest::Client, std::time::Duration};

const DEFAULT_CONCURRENCY: usize = 50;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const URL: &str = "http://treasure-hunt.servegame.com";
const ALL_CLUES: &str = include_str!("../../server/assets/all_clues.json");

mod actors;
mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let all_clues: Vec<serde_json::Value> = serde_json::from_str(ALL_CLUES)?;
    let answers: Vec<(String, String)> = all_clues
        .into_iter()
        .map(|value| {
            (
                value["poem"].as_str().unwrap().into(),
                value["answer"].as_str().unwrap().into(),
            )
        })
        .collect();

    let url = URL;
    let concurrency = DEFAULT_CONCURRENCY;

    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        // Without this, idle connections are capped and later requests queue
        // behind connection reuse instead of actually running in parallel.
        .pool_max_idle_per_host(concurrency)
        .build()?;

    // Warm up DNS + TLS so that cost doesn't land inside the measured window.
    // Failure here is not fatal; the real run will report it.
    let _ = client.get(url).send().await;

    println!("Firing {concurrency} concurrent POSTs at {url}\n");

    let (sender, accumulator) = crate::actors::accumulator::Accumulator::new();

    let handles =
        crate::actors::player::Player::spawn_swarm(concurrency, url, client, sender, answers)?;

    accumulator.spawn().await?;
    for h in handles {
        h.await??;
    }

    Ok(())
}
