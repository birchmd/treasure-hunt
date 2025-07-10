use {
    crate::state::command::Command,
    axum::{
        extract::{Path, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::{clues::status::KnowledgeKind, session::SessionId},
};

pub async fn form(
    State(sender): State<mpsc::Sender<Command>>,
    Path(id): Path<String>,
) -> Html<String> {
    async fn inner_clues_form(
        sender: mpsc::Sender<Command>,
        id: &str,
    ) -> anyhow::Result<Html<String>> {
        let Some(session_id) = SessionId::new(id) else {
            anyhow::bail!("Invalid session ID");
        };
        let (tx, rx) = oneshot::channel();
        let command = Command::GetCurrentClue {
            id: session_id,
            response: tx,
        };
        sender.send(command).await?;
        let maybe_clue = rx.await??;

        let clue_text = match maybe_clue {
            Some((clue, knowledge)) => {
                let mut text = format!("<p>{}</p><br><br>", clue.poem);
                if matches!(
                    knowledge,
                    KnowledgeKind::WithHint | KnowledgeKind::KnowingItem
                ) {
                    text.push_str(&format!("Hint: <p>{}</p><br><br>", clue.hint));
                }
                if matches!(knowledge, KnowledgeKind::KnowingItem) {
                    text.push_str(&format!("Item to find: <p>{}</p><br><br>", clue.item));
                }
                text
            }
            None => {
                return Ok(Html("TODO: All done page".into()));
            }
        };

        // TODO: buttons for getting hints and submitting answer
        Ok(Html(clue_text))
    }

    inner_clues_form(sender, &id)
        .await
        .unwrap_or_else(super::error_to_html)
}
