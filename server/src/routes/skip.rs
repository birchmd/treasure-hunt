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
    treasure_hunt_core::{clues::ClueView, session::SessionId},
};

const MIN_SKIP_DURATION: Duration = Duration::from_secs(10 * 60);

pub async fn action(
    State(sender): State<mpsc::Sender<Command>>,
    Path((session_id, clue_id)): Path<(String, String)>,
) -> Html<String> {
    clues::use_current_clue(sender, &session_id, &clue_id, do_skip)
        .await
        .unwrap_or_else(super::error_to_html)
}

async fn do_skip(
    session_id: SessionId,
    mut clue_view: ClueView,
    sender: mpsc::Sender<Command>,
) -> anyhow::Result<Html<String>> {
    // Require waiting at least 10 minutes before allowing skipping
    if clue_view.duration < MIN_SKIP_DURATION {
        let time_to_hint = MIN_SKIP_DURATION.saturating_sub(clue_view.duration);
        clue_view.clue.poem.push_str(&format!(
            "<br><br>Don't give up yet! Wait at least {} before you can skip.",
            super::format_duration(time_to_hint)
        ));
        return Ok(construct_clues_form(session_id, clue_view));
    }

    let command = Command::SkipClue { id: session_id };
    sender.send(command).await?;
    Ok(clues::form(State(sender), Path(session_id.to_string())).await)
}
