use {
    crate::state::TeamName, axum::response::Html, std::time::Duration,
    treasure_hunt_core::session::SessionId,
};

pub mod answer;
pub mod clues;
pub mod hint;
pub mod leaderboard;
pub mod login;
pub mod register;
pub mod skip;

const BODY_PLACEHOLDER: &str = "${{BODY}}";
const SESSION_ID_PLACEHOLDER: &str = "${{SESSION_ID}}";
const LOGIN_PLACEHOLDER: &str = "${{LOGIN_SECTION}}";

#[derive(Debug)]
pub struct TeamData {
    pub team_name: TeamName,
    pub session_id: SessionId,
}

impl TeamData {
    fn into_html(self) -> String {
        format!(
            r#"<section style="margin-top: 40px;">
                <h4>You are logged in as {}</h4>
                <h4>Your Session ID is {}</h4>
            </section>"#,
            self.team_name, self.session_id,
        )
    }
}

// If the `html-reload` feature is enabled then we read the HTML
// template every time the page is rendered (for ease of debugging).
// Otherwise, the template is statically included in the binary at
// compile-time (for performance).
#[cfg(feature = "html-reload")]
fn fetch_template() -> String {
    let cargo_path = std::path::Path::new(std::env!("CARGO_MANIFEST_DIR"));
    let path = cargo_path.join("html/template.html");
    std::fs::read_to_string(path).unwrap()
}

#[cfg(not(feature = "html-reload"))]
const fn fetch_template() -> &'static str {
    include_str!("../../html/template.html")
}

fn fill_body(content: &str, team_data: Option<TeamData>) -> Html<String> {
    let template = fetch_template();
    let session_id = team_data.as_ref().map(|t| t.session_id.to_string());
    let login_data = team_data.map(TeamData::into_html).unwrap_or_default();
    Html(
        template
            .replace(BODY_PLACEHOLDER, content)
            .replace(LOGIN_PLACEHOLDER, &login_data)
            .replace(
                SESSION_ID_PLACEHOLDER,
                session_id.as_deref().unwrap_or(SESSION_ID_PLACEHOLDER),
            ),
    )
}

fn error_to_html(e: anyhow::Error) -> Html<String> {
    let message = format!("An error occurred: {e:?}");
    fill_body(&message, None)
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
