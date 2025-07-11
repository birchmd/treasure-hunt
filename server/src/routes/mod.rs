use {axum::response::Html, std::time::Duration};

pub mod answer;
pub mod clues;
pub mod hint;
pub mod leaderboard;
pub mod register;
pub mod skip;

const BODY_PLACEHOLDER: &str = "${{BODY}}";

fn fill_body(content: &str) -> Html<String> {
    let template = include_str!("../../html/template.html");
    Html(template.replace(BODY_PLACEHOLDER, content))
}

fn error_to_html(e: anyhow::Error) -> Html<String> {
    let message = format!("An error occurred: {e:?}");
    fill_body(&message)
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    match seconds {
        1 => "1 second".into(),
        seconds if seconds < 60 => format!("{} seconds", seconds),
        seconds => match seconds / 60 {
            1 => "1 minute".into(),
            minutes => format!("{} minutes", minutes),
        },
    }
}
