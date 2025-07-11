use {
    crate::{
        routes::clues::{self, construct_clues_form},
        state::command::Command,
    },
    axum::{
        extract::{Path, State},
        response::Html,
    },
    std::time::Duration,
    tokio::sync::mpsc,
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
    clues::use_current_clue(sender, &session_id, &clue_id, update_with_hint)
        .await
        .unwrap_or_else(super::error_to_html)
}

pub async fn reveal_action(
    State(sender): State<mpsc::Sender<Command>>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    clues::use_current_clue(sender, &session_id, &clue_id, update_with_item)
        .await
        .unwrap_or_else(super::error_to_html)
}

async fn update_with_hint(
    session_id: SessionId,
    mut clue_view: ClueView,
    sender: mpsc::Sender<Command>,
) -> anyhow::Result<Html<String>> {
    // If the current clue does not have the expected level of knowledge
    // then do not make any changes. This could be a spurious request
    // (e.g. from a page reload).
    if !matches!(clue_view.knowledge, KnowledgeKind::Unaided) {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    // Require waiting at least 5 minutes before giving a hint
    if clue_view.duration < MIN_HINT_DURATION {
        let time_to_hint = MIN_HINT_DURATION.saturating_sub(clue_view.duration);
        clue_view.clue.poem.push_str(&format!(
            "<br><br>Wait at least {} for a hint.",
            super::format_duration(time_to_hint)
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
    // If the current clue does not have the expected level of knowledge
    // then do not make any changes. This could be a spurious request
    // (e.g. from a page reload).
    if !matches!(clue_view.knowledge, KnowledgeKind::WithHint) {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    // Require waiting at least 10 minutes before revealing the item
    if clue_view.duration < MIN_ITEM_DURATION {
        let time_to_hint = MIN_ITEM_DURATION.saturating_sub(clue_view.duration);
        clue_view.clue.hint.push_str(&format!(
            "<br><br>Wait at least {} for revealing the item.",
            super::format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, clue_view));
    }

    // Mark clue as revealed
    let command = Command::RevealCurrentItem { id: session_id };
    sender.send(command).await?;
    clue_view.revealed();
    Ok(clues::construct_clues_form(session_id, clue_view))
}
