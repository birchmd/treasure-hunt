use axum::response::Html;

pub mod clues;
pub mod leaderboard;
pub mod register;

fn error_to_html(e: anyhow::Error) -> Html<String> {
    Html(format!("An error occurred: {e:?}"))
}
