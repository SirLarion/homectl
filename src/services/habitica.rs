use clap::Subcommand;
use tokio::runtime::Builder;

use crate::error::AppError;

mod request;
mod types;
mod util;
use util::*;

#[cfg(feature = "tui")]
mod tui;

#[derive(Subcommand)]
pub enum Operation {
    /// List all TODOs
    List {
        /// Save the list of tasks as a JSON file
        #[arg(long, default_value_t = false)]
        save_json: bool,
    },

    /// Create a new TODO item
    Task {
        /// Optionally define TODO item with a descriptor. Format:
        /// <name>,<difficulty>,<notes>,<due>,<checklist1>;<checklist2>;...
        descriptor: Option<String>,
    },

    /// Reorder tasks by descending priority
    Reorder,
}

pub fn run_service(operation: Option<Operation>) -> Result<(), AppError> {
    assert_service_installed()?;

    // Create async runtime to enable fetching Habitica API data
    let runtime = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap();

    let handle = match operation {
        Some(Operation::List { save_json }) => runtime.spawn(list_tasks(save_json)),
        Some(Operation::Task { descriptor }) => runtime.spawn(create_task(descriptor)),
        Some(Operation::Reorder) => runtime.spawn(priority_reorder_tasks()),
        None =>
        {
            #[cfg(feature = "tui")]
            runtime.spawn(tui::run())
        }
    };

    runtime.block_on(handle)??;
    Ok(())
}
