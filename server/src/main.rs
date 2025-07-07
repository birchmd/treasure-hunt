use {
    self::{config::Config, state::Command},
    crate::state::TeamName,
    axum::{
        Router,
        extract::{Form, State},
        handler::Handler,
        response::Html,
        routing::get,
    },
    tokio::sync::{mpsc, oneshot},
    tracing_subscriber::fmt::format::FmtSpan,
};

mod config;
mod state;

fn set_global_tracing_subscriber(config: &Config) {
    tracing_subscriber::fmt()
        .with_max_level(config.log_level.inner)
        .with_span_events(FmtSpan::CLOSE)
        .init();
}

#[tokio::main]
async fn main() {
    let config = Config::read().unwrap();

    set_global_tracing_subscriber(&config);

    tracing::info!(
        "Starting app with config: {}",
        serde_json::to_string(&config).unwrap()
    );

    let (state, sender) = state::State::new(&config).unwrap();

    let state_task = state.spawn();

    let app = Router::new().route(
        "/",
        get(register_form).post_service(Handler::<_, _>::with_state(do_register, sender)),
    );
    let bind_url = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(bind_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    state_task.await.unwrap();
}

async fn register_form() -> Html<&'static str> {
    Html(include_str!("../html/register.html"))
}

async fn do_register(
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
        let id = rx.await?;
        Ok(Html(format!(
            "<html><head></head><body><h1>Welcome</h1><p>Welcome {raw_team_name}! Your session id is {id}.</p></body></html>"
        )))
    }

    inner_register(sender, input)
        .await
        .unwrap_or_else(|e| Html(format!("An error occurred: {e:?}")))
}

#[derive(serde::Deserialize, Debug)]
struct RegisterInput {
    team_name: String,
}
