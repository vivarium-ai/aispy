#![cfg(target_os = "linux")]

use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};

use aispy::{collectors::ebpf::EbpfCollector, entities::EntityResolver};

const TEST_DURATION: Duration = Duration::from_millis(500);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[test]
#[ignore = "requires Linux, a built eBPF object, and privileges to load eBPF programs"]
fn collects_scheduler_events() -> Result<()> {
    let object = ebpf_object();

    ensure!(
        object.is_file(),
        "eBPF object does not exist: {}",
        object.display(),
    );

    let mut collector = EbpfCollector::load(&object)
        .with_context(|| format!("failed to load eBPF object {}", object.display()))?;

    let mut entities = TestEntityResolver;
    let mut events = Vec::new();

    /*
     * Generate some scheduler activity while the eBPF programs are attached.
     * We don't require a particular scheduler event because scheduling
     * decisions are inherently nondeterministic.
     */
    let worker = thread::spawn(|| {
        let deadline = Instant::now() + TEST_DURATION;

        while Instant::now() < deadline {
            thread::yield_now();
        }
    });

    let deadline = Instant::now() + TEST_DURATION;

    while Instant::now() < deadline {
        collector
            .drain(&mut entities, &mut |event| events.push(event))
            .context("failed to drain eBPF events")?;

        thread::sleep(POLL_INTERVAL);
    }

    worker.join().expect("scheduler activity thread panicked");

    /*
     * Drain anything that arrived between the final polling iteration and
     * completion of the worker.
     */
    collector
        .drain(&mut entities, &mut |event| events.push(event))
        .context("failed to perform final eBPF drain")?;

    ensure!(
        !events.is_empty(),
        "eBPF collector produced no scheduler events",
    );

    Ok(())
}

fn ebpf_object() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("ebpf")
        .join("target")
        .join("bpfel-unknown-none")
        .join("release")
        .join("aispy-ebpf")
}

struct TestEntityResolver;

impl EntityResolver for TestEntityResolver {
    fn cpu(&mut self, cpu: u32) -> aispy::Result<u64> {
        Ok(u64::from(cpu))
    }

    fn thread(&mut self, tid: u32) -> aispy::Result<u64> {
        Ok(u64::from(tid))
    }
}
