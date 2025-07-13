use {
    crate::RouteState,
    axum::{
        extract::{Form, Path, State},
        response::Html,
    },
    treasure_hunt_core::session::SessionId,
};

pub async fn action(
    State(route_state): State<RouteState>,
    Form(input): Form<LoginInput>,
) -> Html<String> {
    async fn inner_login(
        route_state: RouteState,
        input: LoginInput,
    ) -> anyhow::Result<Html<String>> {
        // Validate session id
        SessionId::new(&input.session_id)
            .ok_or_else(|| anyhow::Error::msg("Invalid session ID"))?;
        Ok(crate::routes::clues::form(State(route_state), Path(input.session_id)).await)
    }
    inner_login(route_state, input)
        .await
        .unwrap_or_else(super::error_to_html)
}

#[derive(serde::Deserialize, Debug)]
pub struct LoginInput {
    session_id: String,
}
