use {
    crate::{
        RouteState,
        routes::clues::{self, construct_clues_form},
        state::command::Command,
    },
    axum::{
        extract::{Path, State},
        response::Html,
    },
    std::time::Duration,
    treasure_hunt_core::{
        clues::{ClueView, status::KnowledgeKind},
        session::SessionId,
    },
};

pub async fn hint_action(
    State(route_state): State<RouteState>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    clues::use_current_clue(route_state, &session_id, &clue_id, update_with_hint)
        .await
        .unwrap_or_else(super::error_to_html)
}

pub async fn reveal_action(
    State(route_state): State<RouteState>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    clues::use_current_clue(route_state, &session_id, &clue_id, update_with_item)
        .await
        .unwrap_or_else(super::error_to_html)
}

async fn update_with_hint(
    session_id: SessionId,
    mut clue_view: ClueView,
    route_state: RouteState,
) -> anyhow::Result<Html<String>> {
    // If the current clue does not have the expected level of knowledge
    // then do not make any changes. This could be a spurious request
    // (e.g. from a page reload).
    if !matches!(clue_view.knowledge, KnowledgeKind::Unaided) {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    // Require waiting some time before giving a hint
    let min_hint_duration = Duration::from_secs(route_state.config.min_hint_seconds);
    if clue_view.duration < min_hint_duration {
        let time_to_hint = min_hint_duration.saturating_sub(clue_view.duration);
        clue_view.clue.poem.push_str(&format!(
            "<br><br>Wait at least {} for a hint.",
            super::format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, clue_view));
    }

    // Mark clue as hinted
    let command = Command::HintCurrentClue { id: session_id };
    route_state.sender.send(command).await?;
    clue_view.hinted();
    Ok(clues::construct_clues_form(session_id, clue_view))
}

async fn update_with_item(
    session_id: SessionId,
    mut clue_view: ClueView,
    route_state: RouteState,
) -> anyhow::Result<Html<String>> {
    // If the current clue does not have the expected level of knowledge
    // then do not make any changes. This could be a spurious request
    // (e.g. from a page reload).
    if !matches!(clue_view.knowledge, KnowledgeKind::WithHint) {
        return Ok(clues::construct_clues_form(session_id, clue_view));
    }

    // Require waiting some time before revealing the item
    let min_reveal_duration = Duration::from_secs(route_state.config.min_reveal_seconds);
    if clue_view.duration < min_reveal_duration {
        let time_to_hint = min_reveal_duration.saturating_sub(clue_view.duration);
        clue_view.clue.hint.push_str(&format!(
            "<br><br>Wait at least {} for revealing the item.",
            super::format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, clue_view));
    }

    // Mark clue as revealed
    let command = Command::RevealCurrentItem { id: session_id };
    route_state.sender.send(command).await?;
    clue_view.revealed();
    Ok(clues::construct_clues_form(session_id, clue_view))
}
