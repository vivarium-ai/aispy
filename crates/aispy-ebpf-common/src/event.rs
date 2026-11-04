pub const WIRE_ABI_VERSION: u32 = 1;
pub const UNKNOWN_CPU: u32 = u32::MAX;

const PAYLOAD_WORDS: usize = 4;

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    // Scheduler
    SchedSwitch = 1,
    SchedWakeup = 2,
    SchedWakeupNew = 3,
    SchedMigrate = 4,

    // Process/thread lifecycle
    ProcessFork = 100,
    ProcessExec = 101,
    ProcessExit = 102,

    // Interrupts
    IrqEntry = 200,
    IrqExit = 201,
    SoftIrqRaise = 202,
    SoftIrqEntry = 203,
    SoftIrqExit = 204,

    // Kernel workqueues
    WorkqueueQueue = 300,
    WorkqueueStart = 301,
    WorkqueueEnd = 302,

    // Block I/O
    BlockIssue = 400,
    BlockComplete = 401,

    // Memory
    PageFault = 500,
    ReclaimBegin = 501,
    ReclaimEnd = 502,
    OomKill = 503,

    // CPU power
    CpuIdle = 600,
    CpuFrequency = 601,

    // Signals
    SignalGenerate = 700,
    SignalDeliver = 701,

    // Syscalls
    SyscallEnter = 800,
    SyscallExit = 801,

    // Network
    NetRx = 900,
    NetTx = 901,
    TcpRetransmit = 902,
}

impl EventKind {
    #[inline(always)]
    pub const fn from_raw(value: u16) -> Option<Self> {
        match value {
            1 => Some(Self::SchedSwitch),
            2 => Some(Self::SchedWakeup),
            3 => Some(Self::SchedWakeupNew),
            4 => Some(Self::SchedMigrate),

            100 => Some(Self::ProcessFork),
            101 => Some(Self::ProcessExec),
            102 => Some(Self::ProcessExit),

            200 => Some(Self::IrqEntry),
            201 => Some(Self::IrqExit),
            202 => Some(Self::SoftIrqRaise),
            203 => Some(Self::SoftIrqEntry),
            204 => Some(Self::SoftIrqExit),

            300 => Some(Self::WorkqueueQueue),
            301 => Some(Self::WorkqueueStart),
            302 => Some(Self::WorkqueueEnd),

            400 => Some(Self::BlockIssue),
            401 => Some(Self::BlockComplete),

            500 => Some(Self::PageFault),
            501 => Some(Self::ReclaimBegin),
            502 => Some(Self::ReclaimEnd),
            503 => Some(Self::OomKill),

            600 => Some(Self::CpuIdle),
            601 => Some(Self::CpuFrequency),

            700 => Some(Self::SignalGenerate),
            701 => Some(Self::SignalDeliver),

            800 => Some(Self::SyscallEnter),
            801 => Some(Self::SyscallExit),

            900 => Some(Self::NetRx),
            901 => Some(Self::NetTx),
            902 => Some(Self::TcpRetransmit),

            _ => None,
        }
    }
}

/// Fixed-size event transferred from eBPF to userspace.
///
/// Every event is exactly 48 bytes:
///
/// - 8 bytes: timestamp
/// - 4 bytes: CPU
/// - 2 bytes: raw event kind
/// - 2 bytes: reserved
/// - 32 bytes: event-specific payload
///
/// `kind` is stored as a raw `u16` rather than `EventKind` because bytes received
/// from the kernel must not be interpreted as a Rust enum until the discriminant
/// has been validated.
///
/// The payload is deliberately opaque outside this module. Callers construct
/// events with typed constructors and extract them with typed accessors.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct WireEvent {
    pub timestamp_ns: u64,
    pub cpu: u32,
    kind: u16,
    reserved: u16,
    payload: [u64; PAYLOAD_WORDS],
}

impl WireEvent {
    #[inline(always)]
    const fn new(
        timestamp_ns: u64,
        cpu: u32,
        kind: EventKind,
        payload: [u64; PAYLOAD_WORDS],
    ) -> Self {
        Self {
            timestamp_ns,
            cpu,
            kind: kind as u16,
            reserved: 0,
            payload,
        }
    }

    #[inline(always)]
    pub const fn kind_raw(&self) -> u16 {
        self.kind
    }

    #[inline(always)]
    pub const fn kind(&self) -> Option<EventKind> {
        EventKind::from_raw(self.kind)
    }

    // -------------------------------------------------------------------------
    // Scheduler
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn sched_switch(timestamp_ns: u64, cpu: u32, event: SchedSwitch) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SchedSwitch,
            [
                pack_u32(event.prev_tid, event.next_tid),
                event.prev_state as u64,
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn sched_wakeup(timestamp_ns: u64, cpu: u32, event: SchedWakeup) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SchedWakeup,
            [
                pack_u32(event.waker_tid, event.wakee_tid),
                event.target_cpu as u64,
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn sched_wakeup_new(timestamp_ns: u64, cpu: u32, event: SchedWakeup) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SchedWakeupNew,
            [
                pack_u32(event.waker_tid, event.wakee_tid),
                event.target_cpu as u64,
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn sched_migrate(timestamp_ns: u64, cpu: u32, event: SchedMigrate) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SchedMigrate,
            [pack_u32(event.tid, event.destination_cpu), 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_sched_switch(&self) -> Option<SchedSwitch> {
        if !matches!(self.kind(), Some(EventKind::SchedSwitch)) {
            return None;
        }

        let (prev_tid, next_tid) = unpack_u32(self.payload[0]);

        Some(SchedSwitch {
            prev_tid,
            next_tid,
            prev_state: self.payload[1] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_sched_wakeup(&self) -> Option<SchedWakeup> {
        if !matches!(
            self.kind(),
            Some(EventKind::SchedWakeup | EventKind::SchedWakeupNew)
        ) {
            return None;
        }

        let (waker_tid, wakee_tid) = unpack_u32(self.payload[0]);

        Some(SchedWakeup {
            waker_tid,
            wakee_tid,
            target_cpu: self.payload[1] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_sched_migrate(&self) -> Option<SchedMigrate> {
        if !matches!(self.kind(), Some(EventKind::SchedMigrate)) {
            return None;
        }

        let (tid, destination_cpu) = unpack_u32(self.payload[0]);

        Some(SchedMigrate {
            tid,
            destination_cpu,
        })
    }

    // -------------------------------------------------------------------------
    // Process/thread lifecycle
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn process_fork(timestamp_ns: u64, cpu: u32, event: ProcessFork) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::ProcessFork,
            [
                pack_u32(event.parent_tid, event.parent_tgid),
                pack_u32(event.child_tid, event.child_tgid),
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn process_exec(timestamp_ns: u64, cpu: u32, event: ProcessExec) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::ProcessExec,
            [pack_u32(event.tid, event.tgid), 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn process_exit(timestamp_ns: u64, cpu: u32, event: ProcessExit) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::ProcessExit,
            [
                pack_u32(event.tid, event.tgid),
                event.exit_code as u32 as u64,
                0,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn as_process_fork(&self) -> Option<ProcessFork> {
        if !matches!(self.kind(), Some(EventKind::ProcessFork)) {
            return None;
        }

        let (parent_tid, parent_tgid) = unpack_u32(self.payload[0]);
        let (child_tid, child_tgid) = unpack_u32(self.payload[1]);

        Some(ProcessFork {
            parent_tid,
            parent_tgid,
            child_tid,
            child_tgid,
        })
    }

    #[inline(always)]
    pub const fn as_process_exec(&self) -> Option<ProcessExec> {
        if !matches!(self.kind(), Some(EventKind::ProcessExec)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[0]);

        Some(ProcessExec { tid, tgid })
    }

    #[inline(always)]
    pub const fn as_process_exit(&self) -> Option<ProcessExit> {
        if !matches!(self.kind(), Some(EventKind::ProcessExit)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[0]);

        Some(ProcessExit {
            tid,
            tgid,
            exit_code: self.payload[1] as u32 as i32,
        })
    }

    // -------------------------------------------------------------------------
    // IRQ / softirq
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn irq_entry(timestamp_ns: u64, cpu: u32, event: Irq) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::IrqEntry,
            [event.irq as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn irq_exit(timestamp_ns: u64, cpu: u32, event: Irq) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::IrqExit,
            [event.irq as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn soft_irq_raise(timestamp_ns: u64, cpu: u32, event: SoftIrq) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SoftIrqRaise,
            [event.vector as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn soft_irq_entry(timestamp_ns: u64, cpu: u32, event: SoftIrq) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SoftIrqEntry,
            [event.vector as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn soft_irq_exit(timestamp_ns: u64, cpu: u32, event: SoftIrq) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SoftIrqExit,
            [event.vector as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_irq(&self) -> Option<Irq> {
        if !matches!(self.kind(), Some(EventKind::IrqEntry | EventKind::IrqExit)) {
            return None;
        }

        Some(Irq {
            irq: self.payload[0] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_soft_irq(&self) -> Option<SoftIrq> {
        if !matches!(
            self.kind(),
            Some(EventKind::SoftIrqRaise | EventKind::SoftIrqEntry | EventKind::SoftIrqExit)
        ) {
            return None;
        }

        Some(SoftIrq {
            vector: self.payload[0] as u32,
        })
    }

    // -------------------------------------------------------------------------
    // Workqueues
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn workqueue_queue(timestamp_ns: u64, cpu: u32, event: Workqueue) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::WorkqueueQueue,
            [event.work_id, event.function, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn workqueue_start(timestamp_ns: u64, cpu: u32, event: Workqueue) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::WorkqueueStart,
            [event.work_id, event.function, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn workqueue_end(timestamp_ns: u64, cpu: u32, event: Workqueue) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::WorkqueueEnd,
            [event.work_id, event.function, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_workqueue(&self) -> Option<Workqueue> {
        if !matches!(
            self.kind(),
            Some(EventKind::WorkqueueQueue | EventKind::WorkqueueStart | EventKind::WorkqueueEnd)
        ) {
            return None;
        }

        Some(Workqueue {
            work_id: self.payload[0],
            function: self.payload[1],
        })
    }

    // -------------------------------------------------------------------------
    // Block I/O
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn block_issue(timestamp_ns: u64, cpu: u32, event: BlockIo) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::BlockIssue,
            [
                event.request_id,
                event.device,
                event.sector,
                pack_u32(event.bytes, event.operation),
            ],
        )
    }

    #[inline(always)]
    pub const fn block_complete(timestamp_ns: u64, cpu: u32, event: BlockIo) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::BlockComplete,
            [
                event.request_id,
                event.device,
                event.sector,
                pack_u32(event.bytes, event.operation),
            ],
        )
    }

    #[inline(always)]
    pub const fn as_block_io(&self) -> Option<BlockIo> {
        if !matches!(
            self.kind(),
            Some(EventKind::BlockIssue | EventKind::BlockComplete)
        ) {
            return None;
        }

        let (bytes, operation) = unpack_u32(self.payload[3]);

        Some(BlockIo {
            request_id: self.payload[0],
            device: self.payload[1],
            sector: self.payload[2],
            bytes,
            operation,
        })
    }

    // -------------------------------------------------------------------------
    // Memory
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn page_fault(timestamp_ns: u64, cpu: u32, event: PageFault) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::PageFault,
            [
                event.address,
                pack_u32(event.tid, event.tgid),
                event.flags as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn reclaim_begin(timestamp_ns: u64, cpu: u32, event: Reclaim) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::ReclaimBegin,
            [event.pages, pack_u32(event.tid, event.tgid), 0, 0],
        )
    }

    #[inline(always)]
    pub const fn reclaim_end(timestamp_ns: u64, cpu: u32, event: Reclaim) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::ReclaimEnd,
            [event.pages, pack_u32(event.tid, event.tgid), 0, 0],
        )
    }

    #[inline(always)]
    pub const fn oom_kill(timestamp_ns: u64, cpu: u32, event: OomKill) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::OomKill,
            [pack_u32(event.victim_tid, event.victim_tgid), 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_page_fault(&self) -> Option<PageFault> {
        if !matches!(self.kind(), Some(EventKind::PageFault)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[1]);

        Some(PageFault {
            address: self.payload[0],
            tid,
            tgid,
            flags: self.payload[2] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_reclaim(&self) -> Option<Reclaim> {
        if !matches!(
            self.kind(),
            Some(EventKind::ReclaimBegin | EventKind::ReclaimEnd)
        ) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[1]);

        Some(Reclaim {
            pages: self.payload[0],
            tid,
            tgid,
        })
    }

    #[inline(always)]
    pub const fn as_oom_kill(&self) -> Option<OomKill> {
        if !matches!(self.kind(), Some(EventKind::OomKill)) {
            return None;
        }

        let (victim_tid, victim_tgid) = unpack_u32(self.payload[0]);

        Some(OomKill {
            victim_tid,
            victim_tgid,
        })
    }

    // -------------------------------------------------------------------------
    // CPU power
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn cpu_idle(timestamp_ns: u64, cpu: u32, event: CpuIdle) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::CpuIdle,
            [event.state as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn cpu_frequency(timestamp_ns: u64, cpu: u32, event: CpuFrequency) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::CpuFrequency,
            [event.frequency_khz as u64, 0, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_cpu_idle(&self) -> Option<CpuIdle> {
        if !matches!(self.kind(), Some(EventKind::CpuIdle)) {
            return None;
        }

        Some(CpuIdle {
            state: self.payload[0] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_cpu_frequency(&self) -> Option<CpuFrequency> {
        if !matches!(self.kind(), Some(EventKind::CpuFrequency)) {
            return None;
        }

        Some(CpuFrequency {
            frequency_khz: self.payload[0] as u32,
        })
    }

    // -------------------------------------------------------------------------
    // Signals
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn signal_generate(timestamp_ns: u64, cpu: u32, event: Signal) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SignalGenerate,
            [
                pack_u32(event.signal, event.source_tid),
                pack_u32(event.source_tgid, event.target_tid),
                event.target_tgid as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn signal_deliver(timestamp_ns: u64, cpu: u32, event: Signal) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SignalDeliver,
            [
                pack_u32(event.signal, event.source_tid),
                pack_u32(event.source_tgid, event.target_tid),
                event.target_tgid as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn as_signal(&self) -> Option<Signal> {
        if !matches!(
            self.kind(),
            Some(EventKind::SignalGenerate | EventKind::SignalDeliver)
        ) {
            return None;
        }

        let (signal, source_tid) = unpack_u32(self.payload[0]);
        let (source_tgid, target_tid) = unpack_u32(self.payload[1]);

        Some(Signal {
            signal,
            source_tid,
            source_tgid,
            target_tid,
            target_tgid: self.payload[2] as u32,
        })
    }

    // -------------------------------------------------------------------------
    // Syscalls
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn syscall_enter(timestamp_ns: u64, cpu: u32, event: SyscallEnter) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SyscallEnter,
            [pack_u32(event.tid, event.tgid), event.syscall as u64, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn syscall_exit(timestamp_ns: u64, cpu: u32, event: SyscallExit) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::SyscallExit,
            [
                pack_u32(event.tid, event.tgid),
                event.syscall as u64,
                event.result as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn as_syscall_enter(&self) -> Option<SyscallEnter> {
        if !matches!(self.kind(), Some(EventKind::SyscallEnter)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[0]);

        Some(SyscallEnter {
            tid,
            tgid,
            syscall: self.payload[1] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_syscall_exit(&self) -> Option<SyscallExit> {
        if !matches!(self.kind(), Some(EventKind::SyscallExit)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[0]);

        Some(SyscallExit {
            tid,
            tgid,
            syscall: self.payload[1] as u32,
            result: self.payload[2] as i64,
        })
    }

    // -------------------------------------------------------------------------
    // Network
    // -------------------------------------------------------------------------

    #[inline(always)]
    pub const fn net_rx(timestamp_ns: u64, cpu: u32, event: Network) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::NetRx,
            [
                event.socket_id,
                pack_u32(event.tid, event.tgid),
                event.bytes as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn net_tx(timestamp_ns: u64, cpu: u32, event: Network) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::NetTx,
            [
                event.socket_id,
                pack_u32(event.tid, event.tgid),
                event.bytes as u64,
                0,
            ],
        )
    }

    #[inline(always)]
    pub const fn tcp_retransmit(timestamp_ns: u64, cpu: u32, event: TcpRetransmit) -> Self {
        Self::new(
            timestamp_ns,
            cpu,
            EventKind::TcpRetransmit,
            [event.socket_id, event.sequence as u64, 0, 0],
        )
    }

    #[inline(always)]
    pub const fn as_network(&self) -> Option<Network> {
        if !matches!(self.kind(), Some(EventKind::NetRx | EventKind::NetTx)) {
            return None;
        }

        let (tid, tgid) = unpack_u32(self.payload[1]);

        Some(Network {
            socket_id: self.payload[0],
            tid,
            tgid,
            bytes: self.payload[2] as u32,
        })
    }

    #[inline(always)]
    pub const fn as_tcp_retransmit(&self) -> Option<TcpRetransmit> {
        if !matches!(self.kind(), Some(EventKind::TcpRetransmit)) {
            return None;
        }

        Some(TcpRetransmit {
            socket_id: self.payload[0],
            sequence: self.payload[1] as u32,
        })
    }
}

// -----------------------------------------------------------------------------
// Typed payloads
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedSwitch {
    pub prev_tid: u32,
    pub next_tid: u32,
    pub prev_state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedWakeup {
    pub waker_tid: u32,
    pub wakee_tid: u32,
    pub target_cpu: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchedMigrate {
    pub tid: u32,
    pub destination_cpu: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessFork {
    pub parent_tid: u32,
    pub parent_tgid: u32,
    pub child_tid: u32,
    pub child_tgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExec {
    pub tid: u32,
    pub tgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExit {
    pub tid: u32,
    pub tgid: u32,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Irq {
    pub irq: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoftIrq {
    pub vector: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Workqueue {
    pub work_id: u64,
    pub function: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockIo {
    pub request_id: u64,
    pub device: u64,
    pub sector: u64,
    pub bytes: u32,
    pub operation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageFault {
    pub address: u64,
    pub tid: u32,
    pub tgid: u32,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reclaim {
    pub pages: u64,
    pub tid: u32,
    pub tgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OomKill {
    pub victim_tid: u32,
    pub victim_tgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuIdle {
    pub state: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuFrequency {
    pub frequency_khz: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signal {
    pub signal: u32,
    pub source_tid: u32,
    pub source_tgid: u32,
    pub target_tid: u32,
    pub target_tgid: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyscallEnter {
    pub tid: u32,
    pub tgid: u32,
    pub syscall: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyscallExit {
    pub tid: u32,
    pub tgid: u32,
    pub syscall: u32,
    pub result: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Network {
    pub socket_id: u64,
    pub tid: u32,
    pub tgid: u32,
    pub bytes: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpRetransmit {
    pub socket_id: u64,
    pub sequence: u32,
}

// -----------------------------------------------------------------------------
// Wire packing
// -----------------------------------------------------------------------------

#[inline(always)]
const fn pack_u32(low: u32, high: u32) -> u64 {
    (low as u64) | ((high as u64) << 32)
}

#[inline(always)]
const fn unpack_u32(value: u64) -> (u32, u32) {
    (value as u32, (value >> 32) as u32)
}
