use std::{
    mem::size_of,
    path::Path,
    ptr,
};

use anyhow::{bail, Context, Result};
use aya::{
    maps::RingBuf,
    programs::BtfTracePoint,
    Btf,
    Ebpf,
};

use aispy::{
    entities::EntityResolver,
    event::{
        ClockDomain,
        Event,
        EventKind,
        EventSource,
    },
};

const EVENTS_MAP: &str = "EVENTS";

const SCHED_WAKEUP: u32 = 1;
const SCHED_SWITCH: u32 = 2;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct SchedEvent {
    timestamp_ns: u64,

    kind: u32,
    cpu: u32,

    tid: u32,
    related_tid: u32,

    state: u32,
    reserved: u32,
}

pub struct EbpfCollector {
    // Keep the programs and their links alive.
    _ebpf: Ebpf,

    events: RingBuf<aya::maps::MapData>,
}

impl EbpfCollector {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let btf = Btf::from_sys_fs()
            .context("kernel BTF is unavailable at /sys/kernel/btf/vmlinux")?;

        let mut ebpf = Ebpf::load_file(path.as_ref())
            .with_context(|| {
                format!(
                    "failed to load eBPF object {}",
                    path.as_ref().display()
                )
            })?;

        attach_btf_tracepoint(
            &mut ebpf,
            &btf,
            "sched_wakeup",
        )?;

        attach_btf_tracepoint(
            &mut ebpf,
            &btf,
            "sched_switch",
        )?;

        let map = ebpf
            .take_map(EVENTS_MAP)
            .context("eBPF object does not contain EVENTS ring buffer")?;

        let events = RingBuf::try_from(map)
            .context("EVENTS is not a valid BPF ring buffer")?;

        Ok(Self {
            _ebpf: ebpf,
            events,
        })
    }

    pub fn drain(
        &mut self,
        entities: &mut impl EntityResolver,
        emit: &mut impl FnMut(Event),
    ) -> Result<usize> {
        let mut count = 0;

        while let Some(item) = self.events.next() {
            let raw = decode_sched_event(&item)?;

            let event = normalize(raw, entities)?;

            emit(event);

            count += 1;
        }

        Ok(count)
    }

    pub fn ring_buffer(&self) -> &RingBuf<aya::maps::MapData> {
        &self.events
    }
}

fn attach_btf_tracepoint(
    ebpf: &mut Ebpf,
    btf: &Btf,
    name: &str,
) -> Result<()> {
    let program = ebpf
        .program_mut(name)
        .with_context(|| {
            format!("eBPF program {name:?} was not found")
        })?;

    let program: &mut BtfTracePoint = program
        .try_into()
        .with_context(|| {
            format!("{name:?} is not a BTF tracepoint")
        })?;

    program
        .load(name, btf)
        .with_context(|| {
            format!("failed to load BTF tracepoint {name:?}")
        })?;

    program
        .attach()
        .with_context(|| {
            format!("failed to attach BTF tracepoint {name:?}")
        })?;

    Ok(())
}

fn decode_sched_event(bytes: &[u8]) -> Result<SchedEvent> {
    if bytes.len() != size_of::<SchedEvent>() {
        bail!(
            "invalid EVENTS record size: expected {}, received {}",
            size_of::<SchedEvent>(),
            bytes.len()
        );
    }

    /*
     * Do not assume that the byte slice returned by the map is aligned
     * for SchedEvent.
     */
    let event = unsafe {
        ptr::read_unaligned(
            bytes.as_ptr().cast::<SchedEvent>()
        )
    };

    Ok(event)
}

fn normalize(
    raw: SchedEvent,
    entities: &mut impl EntityResolver,
) -> Result<Event> {
    let cpu_entity_id = entities.cpu(raw.cpu)?;
    let thread_entity_id = entities.thread(raw.tid)?;

    let related_thread_entity_id =
        if raw.related_tid == 0 {
            None
        } else {
            Some(entities.thread(raw.related_tid)?)
        };

    let kind = match raw.kind {
        SCHED_WAKEUP => EventKind::SchedulerWakeup,
        SCHED_SWITCH => EventKind::SchedulerSwitch,

        other => {
            bail!("unknown scheduler event kind {other}");
        }
    };

    Ok(Event {
        timestamp: raw.timestamp_ns,
        clock_domain: ClockDomain::HostMonotonic,

        source: EventSource::Ebpf,
        kind,

        entity_id: Some(cpu_entity_id),
        thread_entity_id: Some(thread_entity_id),
        related_entity_id: related_thread_entity_id,

        value_i64: match raw.kind {
            SCHED_SWITCH => Some(i64::from(raw.state)),
            _ => None,
        },

        ..Event::default()
    })
}
