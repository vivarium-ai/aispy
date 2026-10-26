#[derive(Debug, Default)]
pub struct Event {
    pub timestamp: u64,
    pub clock_domain: ClockDomain,

    pub source: EventSource,
    pub kind: EventKind,

    pub entity_id: Option<u64>,
    pub thread_entity_id: Option<u64>,
    pub related_entity_id: Option<u64>,

    pub value_i64: Option<i64>,
}

#[derive(Debug, Default)]
pub enum ClockDomain {
    #[default]
    HostMonotonic,
}

#[derive(Debug, Default)]
pub enum EventSource {
    #[default]
    Ebpf,
}

#[derive(Debug, Default)]
pub enum EventKind {
    #[default]
    SchedulerWakeup,
    SchedulerSwitch,
}
