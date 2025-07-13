use {
    crate::{
        RouteState,
        routes::clues::{self, construct_clues_form},
        state::{TeamName, command::Command},
    },
    axum::{
        extract::{Path, State},
        response::Html,
    },
    std::time::Duration,
    treasure_hunt_core::{clues::ClueView, session::SessionId},
};

pub async fn action(
    State(route_state): State<RouteState>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    clues::use_current_clue(route_state, &session_id, &clue_id, do_skip)
        .await
        .unwrap_or_else(super::error_to_html)
}

async fn do_skip(
    session_id: SessionId,
    team_name: TeamName,
    mut clue_view: ClueView,
    route_state: RouteState,
) -> anyhow::Result<Html<String>> {
    // Require waiting some time before allowing skipping
    let min_skip_duration = Duration::from_secs(route_state.config.min_skip_seconds);
    if clue_view.duration < min_skip_duration {
        let time_to_hint = min_skip_duration.saturating_sub(clue_view.duration);
        clue_view.clue.poem.push_str(&format!(
            "<br><br>Don't give up yet! Wait at least {} before you can skip.",
            super::format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, team_name, clue_view));
    }

    let command = Command::SkipClue { id: session_id };
    route_state.sender.send(command).await?;
    Ok(clues::form(State(route_state), Path(session_id.to_string())).await)
}
