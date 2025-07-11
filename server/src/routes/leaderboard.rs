use {
    crate::{RouteState, state::command::Command},
    axum::{extract::State, response::Html},
    tokio::sync::{mpsc, oneshot},
};

pub async fn action(State(route_state): State<RouteState>) -> Html<String> {
    async fn inner_leaderboard(sender: mpsc::Sender<Command>) -> anyhow::Result<Html<String>> {
        let (tx, rx) = oneshot::channel();
        let command = Command::Leaderboard { response: tx };
        sender.send(command).await?;
        let rows = rx.await?;
        let mut result = String::new();
        result.push_str("<table>\n<tr><th>Team Name</th><th>Score</th></tr>\n");
        for row in rows {
            result.push_str(&format!(
                "<tr><td>{}</td><td>{}</td></tr>\n",
                row.team_name, row.score
            ));
        }
        result.push_str("</table>");
        Ok(super::fill_body(&result))
    }
    inner_leaderboard(route_state.sender)
        .await
        .unwrap_or_else(super::error_to_html)
}
