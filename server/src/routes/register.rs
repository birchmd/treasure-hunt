use {
    crate::state::{TeamName, command::Command},
    axum::{
        extract::{Form, State},
        response::Html,
    },
    tokio::sync::{mpsc, oneshot},
};

pub async fn form() -> Html<String> {
    super::fill_body(include_str!("../../html/register_form.html"))
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
        let html_body = format!(
            r#"<h1>Welcome</h1><p>Welcome {raw_team_name}! Your session id is {id}.</p><br><br><p><a href="/clue/{id}">Click here</a> to see the first clue.</p>"#
        );
        Ok(super::fill_body(&html_body))
    }

    inner_register(sender, input)
        .await
        .unwrap_or_else(super::error_to_html)
}

#[derive(serde::Deserialize, Debug)]
pub struct RegisterInput {
    team_name: String,
}
