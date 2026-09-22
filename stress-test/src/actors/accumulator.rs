use {
    crate::types::Outcome,
    std::time::{Duration, Instant},
    tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

pub struct Accumulator {
    channel: UnboundedReceiver<Outcome>,
}

impl Accumulator {
    pub fn new() -> (UnboundedSender<Outcome>, Self) {
        let (tx, rx) = mpsc::unbounded_channel();
        (tx, Self { channel: rx })
    }

    pub fn spawn(mut self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let wall_clock = Instant::now();
            let mut outcomes = Vec::new();
            while let Some(outcome) = self.channel.recv().await {
                outcomes.push(outcome);
            }
            let total = wall_clock.elapsed();

            outcomes.sort_by_key(|o| o.id);
            for o in &outcomes {
                let status = o
                    .status
                    .map(|s| s.as_u16().to_string())
                    .unwrap_or_else(|| "---".to_string());
                let note = o.error.as_deref().unwrap_or("");
                if o.status.is_some_and(|s| !s.is_success()) || !note.is_empty() {
                    println!(
                        "#{:<4} {:>5}  {:>9.2} ms  {}",
                        o.id,
                        status,
                        ms(o.elapsed),
                        note
                    );
                }
            }

            report(&outcomes, total);
        })
    }
}

fn report(outcomes: &[Outcome], total: Duration) {
    let successes: Vec<Duration> = {
        let mut v: Vec<Duration> = outcomes
            .iter()
            .filter(|o| o.is_success())
            .map(|o| o.elapsed)
            .collect();
        v.sort();
        v
    };

    println!("\n--- summary ---");
    println!("requests:     {}", outcomes.len());
    println!("successful:   {}", successes.len());
    println!("failed:       {}", outcomes.len() - successes.len());
    println!("wall clock:   {:.2} ms", ms(total));

    if successes.is_empty() {
        println!("\nNo successful responses, so no latency stats.");
        return;
    }

    let sum: Duration = successes.iter().sum();
    let mean = sum / successes.len() as u32;

    println!("\nlatency (successful responses only)");
    println!("  min:  {:>9.2} ms", ms(successes[0]));
    println!("  mean: {:>9.2} ms", ms(mean));
    println!("  p50:  {:>9.2} ms", ms(percentile(&successes, 50.0)));
    println!("  p95:  {:>9.2} ms", ms(percentile(&successes, 95.0)));
    println!("  p99:  {:>9.2} ms", ms(percentile(&successes, 99.0)));
    println!("  max:  {:>9.2} ms", ms(successes[successes.len() - 1]));

    // Status code breakdown for anything that wasn't a 2xx.
    let mut non_2xx: Vec<(String, usize)> = Vec::new();
    for o in outcomes.iter().filter(|o| !o.is_success()) {
        let key = match o.status {
            Some(s) => s.as_u16().to_string(),
            None => "transport error".to_string(),
        };
        match non_2xx.iter_mut().find(|(k, _)| *k == key) {
            Some((_, count)) => *count += 1,
            None => non_2xx.push((key, 1)),
        }
    }
    if !non_2xx.is_empty() {
        println!("\nfailures by kind");
        for (kind, count) in non_2xx {
            println!("  {kind}: {count}");
        }
    }
}

/// Nearest-rank percentile over a pre-sorted slice.
fn percentile(sorted: &[Duration], p: f64) -> Duration {
    let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}
