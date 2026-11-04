#[derive(Debug, Default)]
pub struct Event {
    pub timestamp: u64,
    pub clock_domain: ClockDomain,

    pub source: EventSource,
    pub kind: EventKind,

    pub instance_id: Option<u64>,

    pub entity_id: Option<u64>,
    pub thread_entity_id: Option<u64>,
    pub related_entity_id: Option<u64>,

    pub value_i64: Option<i64>,
}

#[derive(Debug, Default)]
pub enum ClockDomain {
    #[default]
    HostMonotonic,
    Device,
}

#[derive(Debug, Default)]
pub enum EventSource {
    #[default]
    Ebpf,
    Semantic,
    Cupti,
    Device,
}

#[derive(Debug, Default)]
pub enum EventKind {
    #[default]
    SchedulerMigrate,
    SchedulerWakeup,
    SchedulerSwitch,

    ObservationBegin,
    ObservationEnd,

    DeviceMarker,
    DeviceRangeBegin,
    DeviceRangeEnd,
}

impl ClockDomain {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::HostMonotonic => "host.monotonic",
            Self::Device => "device",
        }
    }
}

impl EventSource {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ebpf => "ebpf",
            Self::Semantic => "semantic",
            Self::Cupti => "cupti",
            Self::Device => "device",
        }
    }
}

impl EventKind {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SchedulerMigrate => "scheduler.migrate",
            Self::SchedulerWakeup => "scheduler.wakeup",
            Self::SchedulerSwitch => "scheduler.switch",
            Self::ObservationBegin => "observation.begin",
            Self::ObservationEnd => "observation.end",
            Self::DeviceMarker => "device.marker",
            Self::DeviceRangeBegin => "device.range.begin",
            Self::DeviceRangeEnd => "device.range.end",
        }
    }
}
