use std::fmt::{Debug, Display};
use events::configuration::get_configuration;
use tokio::task::JoinError;
use tracing::debug;
use tracing_subscriber;
use events::application::Application;
use events::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("events".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(&configuration).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    debug!("configuration created with values: {:?}", configuration);
    tokio::select! {
        o = application_task => report_exit("API", o)
    }
    Ok(())
}


fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            )
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            )
        }
    }
}
