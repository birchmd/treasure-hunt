use {
    self::config::Config,
    axum::{
        Router,
        handler::Handler,
        routing::{get, post},
    },
    tracing_subscriber::fmt::format::FmtSpan,
};

mod config;
mod routes;
mod state;

fn set_global_tracing_subscriber(config: &Config) {
    tracing_subscriber::fmt()
        .with_max_level(config.log_level.inner)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[tokio::main]
async fn main() {
    let config = Config::read().unwrap();

    set_global_tracing_subscriber(&config);

    tracing::info!(
        "Starting app with config: {}",
        serde_json::to_string(&config).unwrap()
    );

    let (state, sender) = state::State::new(&config).unwrap();

    let state_task = state.spawn();

    let app = Router::new()
        .route(
            "/",
            get(routes::register::form)
                .post_service(routes::register::action.with_state(sender.clone())),
        )
        .route("/leaderboard", get(routes::leaderboard::action))
        .route("/clue/{id}", get(routes::clues::form))
        .route(
            "/hint/{session_id}/{clue_id}",
            post(routes::hint::hint_action),
        )
        .route(
            "/reveal/{session_id}/{clue_id}",
            post(routes::hint::reveal_action),
        )
        .with_state(sender);
    let bind_url = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(bind_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    state_task.await.unwrap();
}
