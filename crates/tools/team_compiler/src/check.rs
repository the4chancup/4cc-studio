//! The `check` command: validation (the structure pass and the deep pass) over every export,
//! reported export by export, with nothing compiled.

use studio_core::{Severity, ToolContext};

use crate::cli::RunInputs;
use crate::events::RunEvents;
use crate::validation::{run_budget, validation_pass};

/// The `check` command: validation, each export reported between its start and its end.
/// Returns the worst severity reported; an exports folder that cannot be read is an error.
pub(crate) fn run(inputs: &RunInputs, ctx: &ToolContext) -> anyhow::Result<Option<Severity>> {
    let pass = validation_pass(inputs, &run_budget(inputs))?;
    let mut events = RunEvents::new(ctx);
    for message in pass.run_messages {
        events.message(message);
    }
    for checked in pass.sources {
        events.started(&checked.source);
        for message in checked.messages {
            events.message(message);
        }
        events.processed(checked.source.export_id);
    }
    Ok(events.worst())
}
