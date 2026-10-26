//! Subcommand implementations.

mod info;
mod mcp;
mod play;
mod render;

use crate::cli::Command;

pub(crate) fn run(command: Command) -> anyhow::Result<()> {
    match command {
        Command::Render {
            input,
            output,
            sample_rate,
        } => render::run(&input, output.as_deref(), sample_rate),
        Command::Info { input } => info::run(&input),
        Command::Play {
            input,
            from_bar,
            looping,
        } => play::run(&input, from_bar, looping),
        Command::Mcp { root, open, http } => mcp::run(root, open, http),
    }
}
