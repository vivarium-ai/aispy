mod cli;
mod collectors;
mod writer;

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use clap::Parser;
use uuid::Uuid;

use aispy::{entities::EntityResolver, event::Event};

use cli::{Cli, Command};
use collectors::{ebpf::EbpfCollector, nvml::NvmlCollector};
use writer::EventWriter;

const EBPF_OBJECT: &str = "bpf/target/bpfel-unknown-none/release/aispy-bpf";

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Run {
            config,
            output,
            command,
        } => run(config, output, command),

        Command::Collect {
            pid,
            config,
            output,
        } => collect(pid, config, output),

        Command::Verify { run } => verify(run),

        Command::Analyze { run } => analyze(run),
    }
}

fn run(_config: Option<PathBuf>, output: Option<PathBuf>, command: Vec<OsString>) -> Result<()> {
    let (run_id, run_dir) = create_run(output)?;

    let mut writer = EventWriter::create(run_dir.join("events.parquet"), run_id)?;

    let mut child = spawn_workload(&command)?;

    let nvml = NvmlCollector::new().ok();

    if let Some(nvml) = &nvml {
        let _device_count = nvml.device_count()?;
    }

    let mut ebpf = load_ebpf();
    let mut entities = RunEntities;

    loop {
        if let Some(collector) = &mut ebpf {
            let mut events = Vec::<Event>::new();

            collector.drain(&mut entities, &mut |event| events.push(event))?;

            writer.write(&events)?;
        }

        if let Some(status) = child.try_wait().context("failed waiting for workload")? {
            writer.close()?;

            if !status.success() {
                anyhow::bail!("workload exited with {status}");
            }

            break;
        }

        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

fn collect(_pid: u32, _config: Option<PathBuf>, output: Option<PathBuf>) -> Result<()> {
    let (run_id, run_dir) = create_run(output)?;

    let writer = EventWriter::create(run_dir.join("events.parquet"), run_id)?;

    writer.close()?;

    Ok(())
}

fn spawn_workload(command: &[OsString]) -> Result<std::process::Child> {
    let (program, args) = command
        .split_first()
        .context("no workload command specified")?;

    std::process::Command::new(program)
        .args(args)
        .spawn()
        .with_context(|| format!("failed to start workload {program:?}"))
}

fn load_ebpf() -> Option<EbpfCollector> {
    let path = Path::new(EBPF_OBJECT);

    if !path.exists() {
        return None;
    }

    match EbpfCollector::load(path) {
        Ok(collector) => Some(collector),

        Err(error) => {
            eprintln!("warning: eBPF collection unavailable: {error}");
            None
        }
    }
}

fn create_run(output: Option<PathBuf>) -> Result<(Uuid, PathBuf)> {
    let run_id = Uuid::now_v7();

    let run_dir = output.unwrap_or_else(|| Path::new("runs").join(run_id.to_string()));

    fs::create_dir_all(&run_dir)
        .with_context(|| format!("failed to create run directory {}", run_dir.display()))?;

    Ok((run_id, run_dir))
}

fn verify(_run: PathBuf) -> Result<()> {
    Ok(())
}

fn analyze(_run: PathBuf) -> Result<()> {
    Ok(())
}

struct RunEntities;

impl EntityResolver for RunEntities {
    fn cpu(&mut self, cpu: u32) -> aispy::Result<u64> {
        Ok(0x1000_0000_0000_0000 | u64::from(cpu))
    }

    fn thread(&mut self, tid: u32) -> aispy::Result<u64> {
        Ok(0x2000_0000_0000_0000 | u64::from(tid))
    }
}
