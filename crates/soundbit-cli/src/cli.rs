//! Command-line arguments.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// SoundBit: 8- and 16-bit music for humans and AI agents.
#[derive(Debug, Parser)]
#[command(name = "soundbit", version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Render a project to a 16-bit WAV file.
    Render {
        /// Project file (.sbproj).
        input: PathBuf,
        /// Output WAV file. Defaults to the input path with a .wav extension.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Sample rate in hertz.
        #[arg(long, default_value_t = 48_000)]
        sample_rate: u32,
    },
    /// Play a project through the default audio device.
    Play {
        /// Project file (.sbproj).
        input: PathBuf,
        /// Bar to start from (counted from 0).
        #[arg(long, default_value_t = 0)]
        from_bar: u16,
        /// Repeat the loop region (or the whole song) until interrupted.
        #[arg(long = "loop")]
        looping: bool,
    },
    /// Show a summary of a project and any chip-profile problems.
    Info {
        /// Project file (.sbproj).
        input: PathBuf,
    },
    /// Run the MCP server on standard input/output, so AI agents can compose music.
    Mcp {
        /// Directory the agent may read and write projects and exports in.
        #[arg(long, default_value = ".")]
        root: PathBuf,
        /// Project to open at start-up. Defaults to a new, empty song.
        #[arg(long)]
        open: Option<PathBuf>,
        /// Serve streamable HTTP on 127.0.0.1 at this port instead of stdio.
        #[arg(long, value_name = "PORT")]
        http: Option<u16>,
    },
}
