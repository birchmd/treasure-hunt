use axum::response::Html;

pub mod answer;
pub mod clues;
pub mod hint;
pub mod leaderboard;
pub mod register;

const BODY_PLACEHOLDER: &str = "${{BODY}}";

fn fill_body(content: &str) -> Html<String> {
    let template = include_str!("../../html/template.html");
    Html(template.replace(BODY_PLACEHOLDER, content))
}

fn error_to_html(e: anyhow::Error) -> Html<String> {
    let message = format!("An error occurred: {e:?}");
    fill_body(&message)
}
