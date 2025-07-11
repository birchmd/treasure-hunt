use {
    crate::{
        RouteState,
        routes::clues::{self, construct_clues_form},
        state::command::Command,
    },
    axum::{
        extract::{Form, Path, State},
        response::Html,
    },
    tokio::sync::oneshot,
    treasure_hunt_core::{clues::ClueView, session::SessionId},
};

pub async fn action(
    State(route_state): State<RouteState>,
    Path((session_id, clue_id)): Path<(String, String)>,
    Form(guess): Form<AnswerInput>,
) -> Html<String> {
    let logic = |session_id, mut clue_view: ClueView, route_state: RouteState| async move {
        let (tx, rx) = oneshot::channel();
        let command = Command::AnswerCurrentClue {
            id: session_id,
            guess: guess.clue_answer,
            response: tx,
        };
        route_state.sender.send(command).await?;
        let points = rx.await??;
        match points {
            None => {
                clue_view
                    .clue
                    .poem
                    .push_str("<br><br>That's the wrong answer! Try again.");
                Ok(construct_clues_form(session_id, clue_view))
            }
            Some(x) if x >= 0 => Ok(super::fill_body(&correct_answer(session_id))),
            Some(penalty) => {
                let message = format!(
                    "<br><br>That answer is correct for <em>some</em> clue, but not <em>this</em> clue. You lose {} points for your error. Try again to find the answer for the current clue.",
                    penalty.abs()
                );
                clue_view.clue.poem.push_str(&message);
                Ok(construct_clues_form(session_id, clue_view))
            }
        }
    };
    clues::use_current_clue(route_state, &session_id, &clue_id, logic)
        .await
        .unwrap_or_else(super::error_to_html)
}

#[derive(serde::Deserialize, Debug)]
pub struct AnswerInput {
    clue_answer: String,
}

fn correct_answer(session_id: SessionId) -> String {
    format!(
        r#"<p>Great job! You got the right answer! <a href="/clue/{session_id}">Click here</a> to see the next clue.</p>"#
    )
}
