use {
    crate::{RouteState, state::command::Command},
    axum::{
        extract::{Path, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
    treasure_hunt_core::session::SessionId,
};

pub async fn action(
    State(route_state): State<RouteState>,
    Path(session_id): Path<String>,
) -> Html<String> {
    async fn inner_leaderboard(
        sender: mpsc::Sender<Command>,
        session_id: String,
    ) -> anyhow::Result<Html<String>> {
        let (tx, rx) = oneshot::channel();
        let command = Command::Leaderboard {
            maybe_id: session_id.clone(),
            response: tx,
        };
        sender.send(command).await?;
        let (rows, team_name) = rx.await?;
        let mut result = String::new();
        result.push_str("<table>\n<tr><th>Team Name</th><th>Score</th></tr>\n");
        for row in rows {
            if Some(&row.team_name) == team_name.as_ref() {
                result.push_str(&format!(
                    "<tr><td><b>{}</b></td><td><b>{}</b></td></tr>\n",
                    row.team_name, row.score
                ));
            } else {
                result.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td></tr>\n",
                    row.team_name, row.score
                ));
            }
        }
        result.push_str("</table>");
        let team_data = team_name.and_then(|name| {
            Some(super::TeamData {
                team_name: name,
                session_id: SessionId::new(&session_id)?,
            })
        });
        Ok(super::fill_body(&result, team_data))
    }
    inner_leaderboard(route_state.sender, session_id)
        .await
        .unwrap_or_else(super::error_to_html)
}
