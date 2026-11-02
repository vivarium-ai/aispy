mod cli;
mod writer;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Run { .. } => run(),
        Command::Collect { .. } => collect(),
        Command::Verify { .. } => verify(),
        Command::Analyze { .. } => analyze(),
    }
}

fn run() {}
fn collect() {}
fn verify() {}
fn analyze() {}
