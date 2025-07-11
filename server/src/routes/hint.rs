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
    treasure_hunt_core::{
        clues::{ClueView, status::KnowledgeKind},
        session::SessionId,
    },
};

const MIN_HINT_DURATION: Duration = Duration::from_secs(5 * 60);
const MIN_ITEM_DURATION: Duration = Duration::from_secs(10 * 60);

pub async fn hint_action(
    State(sender): State<mpsc::Sender<Command>>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    common_action(sender, &session_id, &clue_id, KnowledgeKind::Unaided)
        .await
        .unwrap_or_else(super::error_to_html)
}

pub async fn reveal_action(
    State(sender): State<mpsc::Sender<Command>>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    common_action(sender, &session_id, &clue_id, KnowledgeKind::WithHint)
        .await
        .unwrap_or_else(super::error_to_html)
}

async fn common_action(
    sender: mpsc::Sender<Command>,
    session_id: &str,
    clue_id: &str,
    expected_knowledge: KnowledgeKind,
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
    let Some(clue_view) = rx.await?? else {
        return Ok(no_more_clues());
    };

    // If the current clue does not match the one the hint was
    // requested for then the request is invalid and we just show
    // the normal clues page.
    if hex::encode(clue_view.clue.code) != clue_id {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    // If the current clue does not have the expected level of knowledge
    // then do not make any changes. This could be a spurious request
    // (e.g. from a page reload).
    if clue_view.knowledge != expected_knowledge {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    match &clue_view.knowledge {
        KnowledgeKind::Unaided => update_with_hint(session_id, clue_view, sender).await,
        KnowledgeKind::WithHint => update_with_item(session_id, clue_view, sender).await,
        KnowledgeKind::KnowingItem => Ok(clues::construct_clues_form(session_id, clue_view)),
    }
}

async fn update_with_hint(
    session_id: SessionId,
    mut clue_view: ClueView,
    sender: mpsc::Sender<Command>,
) -> anyhow::Result<Html<String>> {
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

async fn update_with_item(
    session_id: SessionId,
    mut clue_view: ClueView,
    sender: mpsc::Sender<Command>,
) -> anyhow::Result<Html<String>> {
    // Require waiting at least 10 minutes before revealing the item
    if clue_view.duration < MIN_ITEM_DURATION {
        let time_to_hint = MIN_ITEM_DURATION.saturating_sub(clue_view.duration);
        clue_view.clue.hint.push_str(&format!(
            "<br><br>Wait at least {} for revealing the item.",
            format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, clue_view));
    }

    // Mark clue as revealed
    let command = Command::RevealCurrentItem { id: session_id };
    sender.send(command).await?;
    clue_view.revealed();
    Ok(clues::construct_clues_form(session_id, clue_view))
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
