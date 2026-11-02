use std::{ffi::OsString, path::PathBuf};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "aispy",
    version,
    about = "Performance instrumentation and analysis for heterogeneous workloads"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Launch a workload under AiSpy and capture observations.
    ///
    /// AiSpy starts the target process, enables the configured collectors,
    /// records observations, and writes the resulting run data.
    Run {
        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(required = true, trailing_var_arg = true)]
        command: Vec<OsString>,
    },

    /// Attach to an already-running process and begin collecting observations.
    ///
    /// Use this when the workload was started without `aispy run` and you want
    /// to begin observation without restarting it.
    Collect {
        #[arg(long)]
        pid: u32,

        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Verify the integrity and internal consistency of a captured run.
    ///
    /// Checks the run metadata, schemas, files, and recorded integrity data.
    Verify { run: PathBuf },

    /// Analyze a completed run for performance bottlenecks.
    ///
    /// Examines captured observations to identify critical-path delays,
    /// resource bottlenecks, contention, queueing, and other throughput limits.
    Analyze { run: PathBuf },
}
