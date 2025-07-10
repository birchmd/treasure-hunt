use {
    crate::state::command::Command,
    axum::{
        extract::{Path, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::{
        clues::{ClueView, status::KnowledgeKind},
        session::SessionId,
    },
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
        let Some(clue_view) = rx.await?? else {
            return Ok(no_more_clues());
        };
        Ok(construct_clues_form(session_id, clue_view))
    }

    inner_clues_form(sender, &id)
        .await
        .unwrap_or_else(super::error_to_html)
}

pub fn construct_clues_form(session_id: SessionId, clue_view: ClueView) -> Html<String> {
    let clue = clue_view.clue;
    let knowledge = clue_view.knowledge;
    let skip_text = if clue_view.is_previously_skipped {
        "Skip forever"
    } else {
        "Skip for now"
    };

    let mut html_body = format!("<p>{}</p><br><br>\n", clue.poem);

    if matches!(
        knowledge,
        KnowledgeKind::WithHint | KnowledgeKind::KnowingItem
    ) {
        html_body.push_str(&format!("Hint: <p>{}</p><br><br>\n", clue.hint));
    }

    if matches!(knowledge, KnowledgeKind::KnowingItem) {
        html_body.push_str(&format!("Item to find: <p>{}</p><br><br>\n", clue.item));
    }

    if matches!(knowledge, KnowledgeKind::Unaided) {
        html_body.push_str(include_str!("../../html/hint_form.html"));
    }

    html_body.push_str(include_str!("../../html/answer_form.html"));
    html_body.push_str(include_str!("../../html/skip_form.html"));

    let html_body = html_body
        .replace("${{SESSION_ID}}", &session_id.to_string())
        .replace("${{CLUE_ID}}", &hex::encode(clue.code))
        .replace("${{SKIP_BUTTON_TEXT}}", skip_text);

    super::fill_body(&html_body)
}

pub fn no_more_clues() -> Html<String> {
    Html("TODO: All done page".into())
}
