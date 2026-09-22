use {
    crate::state::TeamName, axum::response::Html, std::time::Duration,
    treasure_hunt_core::session::SessionId,
};

pub mod about;
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
const COUTNDOWN_START_PLACEHOLDER: &str = "${{COUNTDOWN_START}}";
const COUTNDOWN_MSG_PLACEHOLDER: &str = "${{COUNTDOWN_MSG}}";
const JS_COUNTDOWN: &str = r#"<script>
  const COUNTDOWN_START_SECONDS = ${{COUNTDOWN_START}};

  const countdownEl = document.getElementById('countdown');

  // Compute a fixed end time once, based on when the page loaded
  const endTime = Date.now() + COUNTDOWN_START_SECONDS * 1000;

  function formatTime(totalSeconds) {
    const mins = Math.floor(totalSeconds / 60);
    const secs = totalSeconds % 60;
    return String(mins).padStart(2, '0') + ':' + String(secs).padStart(2, '0');
  }

  let timerId;

  function tick() {
    const remainingMs = endTime - Date.now();
    const remainingSeconds = Math.max(Math.ceil(remainingMs / 1000), 0);

    countdownEl.textContent = '${{COUNTDOWN_MSG}} ' + formatTime(remainingSeconds);

    if (remainingSeconds <= 0) {
      clearInterval(timerId);
    }
  }

  tick(); // render immediately
  timerId = setInterval(tick, 1000);

  // Re-sync immediately when the page becomes visible again
  // (covers the case where setInterval was throttled/paused while hidden)
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      tick();
    }
  });
</script>"#;

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

fn countdown_function(msg: &str, duration: Duration) -> String {
    JS_COUNTDOWN
        .replace(COUTNDOWN_START_PLACEHOLDER, &duration.as_secs().to_string())
        .replace(COUTNDOWN_MSG_PLACEHOLDER, msg)
}
