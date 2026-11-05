use std::{mem::size_of, path::Path, ptr};

use aispy_ebpf_common::{
    EventKind as WireEventKind, SchedMigrate, SchedSwitch, SchedWakeup, WireEvent,
};
use anyhow::{Context, Result, bail};
use aya::{
    Btf, Ebpf,
    maps::{MapData, RingBuf},
    programs::BtfTracePoint,
};

use crate::{
    entities::EntityResolver,
    event::{ClockDomain, Event, EventKind, EventSource},
};

const EVENTS_MAP: &str = "EVENTS";

const SCHEDULER_PROGRAMS: &[&str] = &[
    "sched_wakeup",
    "sched_wakeup_new",
    "sched_switch",
    "sched_migrate_task",
];

pub struct EbpfCollector {
    // The loaded eBPF object owns the programs and attached links. It must
    // therefore remain alive for as long as the collector is active.
    _ebpf: Ebpf,

    events: RingBuf<MapData>,
}

impl EbpfCollector {
    /// Load the AiSpy eBPF object, attach its scheduler tracepoints, and open
    /// the shared EVENTS ring buffer.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let btf =
            Btf::from_sys_fs().context("kernel BTF is unavailable at /sys/kernel/btf/vmlinux")?;

        let mut ebpf = Ebpf::load_file(path)
            .with_context(|| format!("failed to load eBPF object {}", path.display()))?;

        for name in SCHEDULER_PROGRAMS {
            attach_btf_tracepoint(&mut ebpf, &btf, name)?;
        }

        let map = ebpf
            .take_map(EVENTS_MAP)
            .with_context(|| format!("eBPF object does not contain {EVENTS_MAP:?} ring buffer"))?;

        let events = RingBuf::try_from(map)
            .with_context(|| format!("{EVENTS_MAP:?} is not a valid BPF ring buffer"))?;

        Ok(Self {
            _ebpf: ebpf,
            events,
        })
    }

    /// Drain all records currently available from the eBPF ring buffer.
    ///
    /// Each wire record is copied out of Aya's ring-buffer storage before it
    /// is normalized. The callback receives AiSpy's semantic Event type, not
    /// the eBPF wire representation.
    pub fn drain(
        &mut self,
        entities: &mut impl EntityResolver,
        emit: &mut impl FnMut(Event),
    ) -> Result<usize> {
        let mut count = 0;

        while let Some(item) = self.events.next() {
            let wire = decode_wire_event(&item)?;
            let event = normalize(wire, entities)?;

            emit(event);
            count += 1;
        }

        Ok(count)
    }
}

fn attach_btf_tracepoint(ebpf: &mut Ebpf, btf: &Btf, name: &str) -> Result<()> {
    let program = ebpf
        .program_mut(name)
        .with_context(|| format!("eBPF program {name:?} was not found"))?;

    let program: &mut BtfTracePoint = program
        .try_into()
        .with_context(|| format!("{name:?} is not a BTF tracepoint"))?;

    program
        .load(name, btf)
        .with_context(|| format!("failed to load BTF tracepoint {name:?}"))?;

    program
        .attach()
        .with_context(|| format!("failed to attach BTF tracepoint {name:?}"))?;

    Ok(())
}

/// Copy one ring-buffer record into the shared wire representation.
///
/// Aya exposes a ring-buffer item as bytes whose address must not be assumed
/// to satisfy `WireEvent`'s alignment. We therefore verify the ABI size and
/// perform an unaligned copy.
///
/// This is the only place where raw kernel-provided bytes become a Rust
/// `WireEvent`. `WireEvent` deliberately stores its event kind as `u16`, so
/// every possible 48-byte input is a valid Rust value. The discriminant is
/// validated later by `WireEvent::kind()`.
fn decode_wire_event(bytes: &[u8]) -> Result<WireEvent> {
    if bytes.len() != size_of::<WireEvent>() {
        bail!(
            "invalid {EVENTS_MAP} record size: expected {}, received {}",
            size_of::<WireEvent>(),
            bytes.len(),
        );
    }

    // SAFETY:
    //
    // - `bytes` contains at least `size_of::<WireEvent>()` bytes, checked
    //   above.
    // - `read_unaligned` does not require the source address to satisfy
    //   `WireEvent`'s alignment.
    // - WireEvent contains only integer values and an integer array, so every
    //   possible bit pattern is valid.
    // - The value is copied out of the ring buffer; no reference into Aya's
    //   borrowed storage escapes this function.
    Ok(unsafe { ptr::read_unaligned(bytes.as_ptr().cast::<WireEvent>()) })
}

fn normalize(wire: WireEvent, entities: &mut impl EntityResolver) -> Result<Event> {
    let kind = wire
        .kind()
        .with_context(|| format!("unknown eBPF wire event kind {}", wire.kind_raw(),))?;

    match kind {
        WireEventKind::SchedSwitch => {
            let payload = wire
                .as_sched_switch()
                .context("SchedSwitch wire event has an invalid payload")?;

            normalize_sched_switch(wire, payload, entities)
        }

        WireEventKind::SchedWakeup | WireEventKind::SchedWakeupNew => {
            let payload = wire
                .as_sched_wakeup()
                .context("SchedWakeup wire event has an invalid payload")?;

            normalize_sched_wakeup(wire, payload, entities)
        }

        WireEventKind::SchedMigrate => {
            let payload = wire
                .as_sched_migrate()
                .context("SchedMigrate wire event has an invalid payload")?;

            normalize_sched_migrate(wire, payload, entities)
        }

        other => {
            bail!("unsupported eBPF wire event kind {other:?}");
        }
    }
}

fn normalize_sched_switch(
    wire: WireEvent,
    payload: SchedSwitch,
    entities: &mut impl EntityResolver,
) -> Result<Event> {
    let cpu_entity_id = entities.cpu(wire.cpu)?;
    let prev_thread_entity_id = entities.thread(payload.prev_tid)?;
    let next_thread_entity_id = entities.thread(payload.next_tid)?;

    Ok(Event {
        timestamp: wire.timestamp_ns,
        clock_domain: ClockDomain::HostMonotonic,

        source: EventSource::Ebpf,
        kind: EventKind::SchedulerSwitch,

        entity_id: Some(cpu_entity_id),
        thread_entity_id: Some(prev_thread_entity_id),
        related_entity_id: Some(next_thread_entity_id),

        value_i64: Some(i64::from(payload.prev_state)),

        ..Event::default()
    })
}

fn normalize_sched_wakeup(
    wire: WireEvent,
    payload: SchedWakeup,
    entities: &mut impl EntityResolver,
) -> Result<Event> {
    let cpu_entity_id = entities.cpu(wire.cpu)?;
    let wakee_thread_entity_id = entities.thread(payload.wakee_tid)?;

    let waker_thread_entity_id = if payload.waker_tid == 0 {
        None
    } else {
        Some(entities.thread(payload.waker_tid)?)
    };

    Ok(Event {
        timestamp: wire.timestamp_ns,
        clock_domain: ClockDomain::HostMonotonic,

        source: EventSource::Ebpf,
        kind: EventKind::SchedulerWakeup,

        entity_id: Some(cpu_entity_id),
        thread_entity_id: Some(wakee_thread_entity_id),
        related_entity_id: waker_thread_entity_id,

        value_i64: if payload.target_cpu == u32::MAX {
            None
        } else {
            Some(i64::from(payload.target_cpu))
        },

        ..Event::default()
    })
}

fn normalize_sched_migrate(
    wire: WireEvent,
    payload: SchedMigrate,
    entities: &mut impl EntityResolver,
) -> Result<Event> {
    let cpu_entity_id = entities.cpu(wire.cpu)?;
    let thread_entity_id = entities.thread(payload.tid)?;

    Ok(Event {
        timestamp: wire.timestamp_ns,
        clock_domain: ClockDomain::HostMonotonic,

        source: EventSource::Ebpf,
        kind: EventKind::SchedulerMigrate,

        entity_id: Some(cpu_entity_id),
        thread_entity_id: Some(thread_entity_id),
        related_entity_id: None,

        value_i64: Some(i64::from(payload.destination_cpu)),

        ..Event::default()
    })
}
