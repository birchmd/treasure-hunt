use {
    crate::state::{TeamName, command::Command},
    axum::{
        extract::{Form, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
};

pub async fn form() -> Html<&'static str> {
    Html(include_str!("../../html/register.html"))
}

pub async fn action(
    State(sender): State<mpsc::Sender<Command>>,
    Form(input): Form<RegisterInput>,
) -> Html<String> {
    async fn inner_register(
        sender: mpsc::Sender<Command>,
        input: RegisterInput,
    ) -> anyhow::Result<Html<String>> {
        let raw_team_name = &input.team_name;
        let team_name = TeamName::new(raw_team_name)?;
        let (tx, rx) = oneshot::channel();
        let command = Command::NewSession {
            team_name,
            response: tx,
        };
        sender.send(command).await?;
        let id = rx.await??;
        Ok(Html(format!(
            "<html><head></head><body><h1>Welcome</h1><p>Welcome {raw_team_name}! Your session id is {id}.</p></body></html>"
        )))
    }

    inner_register(sender, input)
        .await
        .unwrap_or_else(|e| Html(format!("An error occurred: {e:?}")))
}

#[derive(serde::Deserialize, Debug)]
pub struct RegisterInput {
    team_name: String,
}
