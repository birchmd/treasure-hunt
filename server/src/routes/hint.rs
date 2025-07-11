use {
    crate::{
        routes::clues::{self, construct_clues_form, no_more_clues},
        state::command::Command,
    },
    axum::{
        extract::{Path, State},
        response::Html,
    },
    std::time::Duration,
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::session::SessionId,
};

const MIN_HINT_DURATION: Duration = Duration::from_secs(5 * 60);

pub async fn action(
    State(sender): State<mpsc::Sender<Command>>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    async fn inner(
        sender: mpsc::Sender<Command>,
        session_id: &str,
        clue_id: &str,
    ) -> anyhow::Result<Html<String>> {
        let Some(session_id) = SessionId::new(session_id) else {
            anyhow::bail!("Invalid session ID");
        };

        // Look up current clue
        let (tx, rx) = oneshot::channel();
        let command = Command::GetCurrentClue {
            id: session_id,
            response: tx,
        };
        sender.send(command).await?;
        let Some(mut clue_view) = rx.await?? else {
            return Ok(no_more_clues());
        };

        // If the current clue does not match the one the hint was
        // requested for then the request is invalid and we just show
        // the normal clues page.
        if hex::encode(clue_view.clue.code) != clue_id {
            return Ok(clues::construct_clues_form(session_id, clue_view));
        }

        // Require waiting at least 5 minutes before giving a hint
        if clue_view.duration < MIN_HINT_DURATION {
            let time_to_hint = MIN_HINT_DURATION.saturating_sub(clue_view.duration);
            clue_view.clue.poem.push_str(&format!(
                "<br><br>Wait at least {} for a hint.",
                format_duration(time_to_hint)
            ));
            return Ok(construct_clues_form(session_id, clue_view));
        }

        // Mark clue as hinted
        let command = Command::HintCurrentClue { id: session_id };
        sender.send(command).await?;
        clue_view.hinted();
        Ok(clues::construct_clues_form(session_id, clue_view))
    }

    inner(sender, &session_id, &clue_id)
        .await
        .unwrap_or_else(super::error_to_html)
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
