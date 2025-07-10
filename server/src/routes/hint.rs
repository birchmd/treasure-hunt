use {
    crate::{
        routes::clues::{self, no_more_clues},
        state::command::Command,
    },
    axum::{
        extract::{Path, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::session::SessionId,
};

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
