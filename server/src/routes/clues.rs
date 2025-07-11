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

pub async fn use_current_clue<G, F>(
    sender: mpsc::Sender<Command>,
    session_id: &str,
    clue_id: &str,
    logic: G,
) -> anyhow::Result<Html<String>>
where
    F: Future<Output = anyhow::Result<Html<String>>>,
    G: FnOnce(SessionId, ClueView, mpsc::Sender<Command>) -> F,
{
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
    let Some(clue_view) = rx.await?? else {
        return Ok(no_more_clues());
    };

    // If the current clue does not match the one the hint was
    // requested for then the request is invalid and we just show
    // the normal clues page.
    if hex::encode(clue_view.clue.code) != clue_id {
        return Ok(construct_clues_form(session_id, clue_view));
    }

    logic(session_id, clue_view, sender).await
}

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
    let (hint_url, hint_button_text) = match knowledge {
        KnowledgeKind::Unaided => ("hint", "Ask for a hint"),
        KnowledgeKind::WithHint => ("reveal", "Reveal the item to find"),
        KnowledgeKind::KnowingItem => ("null", "NULL"),
    };
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

    if matches!(knowledge, KnowledgeKind::Unaided | KnowledgeKind::WithHint) {
        html_body.push_str(include_str!("../../html/hint_form.html"));
    }

    html_body.push_str(include_str!("../../html/answer_form.html"));
    html_body.push_str(include_str!("../../html/skip_form.html"));

    let html_body = html_body
        .replace("${{SESSION_ID}}", &session_id.to_string())
        .replace("${{CLUE_ID}}", &hex::encode(clue.code))
        .replace("${{HINT_BASE_URL}}", hint_url)
        .replace("${{HINT_BUTTON_TEXT}}", hint_button_text)
        .replace("${{SKIP_BUTTON_TEXT}}", skip_text);

    super::fill_body(&html_body)
}

pub fn no_more_clues() -> Html<String> {
    Html("TODO: All done page".into())
}

#[test]
fn test_construct_clues_form() {
    let session_id = SessionId::random();
    let clue = treasure_hunt_core::clues::Clue::mock(1, "A");
    let duration = std::time::Duration::from_secs(0);
    let mut clue_view = ClueView::new(clue, KnowledgeKind::Unaided, false, duration);

    let text = construct_clues_form(session_id, clue_view.clone()).0;
    assert!(
        !text.contains("Hint: <p>"),
        "Hint is NOT present in unaided clue"
    );
    assert!(
        !text.contains("Item to find: <p>"),
        "Revealed item is NOT present in unaided clue"
    );
    assert!(
        text.contains(r#"<input type="submit" value="Ask for a hint">"#)
            && text.contains("<form action=\"/hint/"),
        "Hint button is present for unaided clue"
    );
    assert!(
        text.contains(r#"<input type="submit" value="Skip for now">"#),
        "skip button is present for unskipped clue"
    );

    clue_view.hinted();
    let text = construct_clues_form(session_id, clue_view.clone()).0;
    assert!(text.contains("Hint: <p>"), "Hint is present in hinted clue");
    assert!(
        !text.contains("Item to find: <p>"),
        "Revealed item is NOT present in hinted clue"
    );
    assert!(
        text.contains(r#"<input type="submit" value="Reveal the item to find">"#)
            && text.contains("<form action=\"/reveal/"),
        "Reveal button is present for hinted clue:\n{text}",
    );
    assert!(
        text.contains(r#"<input type="submit" value="Skip for now">"#),
        "skip button is present for unskipped clue"
    );

    clue_view.revealed();
    let text = construct_clues_form(session_id, clue_view.clone()).0;
    assert!(
        text.contains("Hint: <p>"),
        "Hint is present in revealed clue"
    );
    assert!(
        text.contains("Item to find: <p>"),
        "Revealed item is present in revealed clue"
    );
    assert!(
        !text.contains(r#"<input type="submit" value="NULL">"#),
        "Hint button is hidden for revealed clue"
    );
    assert!(
        text.contains(r#"<input type="submit" value="Skip for now">"#),
        "skip button is present for unskipped clue"
    );

    clue_view.is_previously_skipped = true;
    let text = construct_clues_form(session_id, clue_view.clone()).0;
    assert!(
        text.contains(r#"<input type="submit" value="Skip forever">"#),
        "skip forever button is present for previously skipped clue"
    );
}
