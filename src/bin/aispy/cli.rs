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
    Run {
        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,

        #[arg(required = true, trailing_var_arg = true)]
        command: Vec<OsString>,
    },

    Collect {
        #[arg(long)]
        pid: u32,

        #[arg(short, long)]
        config: Option<PathBuf>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Verify {
        run: PathBuf,
    },

    Analyze {
        run: PathBuf,
    },
}
