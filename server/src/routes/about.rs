use {
    crate::{RouteState, routes::TeamData, state::command::Command},
    axum::{
        extract::{Path, State},
        response::Html,
    },
    std::time::Duration,
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::session::SessionId,
};

pub async fn action(State(route_state): State<RouteState>, Path(id): Path<String>) -> Html<String> {
    let team_data = lookup_team_data(&id, route_state.sender).await;
    let min_skip_duration = Duration::from_secs(route_state.config.min_skip_seconds);
    let content = fetch_content().replace(
        "${{MIN_SKIP_DURATION}}",
        &super::format_duration(min_skip_duration),
    );
    super::fill_body(&content, team_data)
}

#[cfg(feature = "html-reload")]
fn fetch_content() -> String {
    let cargo_path = std::path::Path::new(std::env!("CARGO_MANIFEST_DIR"));
    let path = cargo_path.join("html/about.html");
    std::fs::read_to_string(path).unwrap()
}

#[cfg(not(feature = "html-reload"))]
const fn fetch_content() -> &'static str {
    include_str!("../../html/about.html")
}

async fn lookup_team_data(maybe_id: &str, sender: mpsc::Sender<Command>) -> Option<TeamData> {
    let session_id = SessionId::new(maybe_id)?;
    let (tx, rx) = oneshot::channel();
    let command = Command::GetCurrentClue {
        id: session_id,
        response: tx,
    };
    sender.send(command).await.ok()?;
    let (team_name, _) = rx.await.ok()?.ok()?;
    Some(TeamData {
        team_name,
        session_id,
    })
}
