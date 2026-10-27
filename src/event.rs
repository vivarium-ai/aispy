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
    SchedulerWakeup,

    SchedulerSwitch,

    ObservationBegin,
    ObservationEnd,

    DeviceMarker,
    DeviceRangeBegin,
    DeviceRangeEnd,
}
