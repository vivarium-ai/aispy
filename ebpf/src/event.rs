#[repr(u32)]
#[derive(Clone, Copy)]
pub enum SchedEventKind {
    Wakeup = 1,
    Switch = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SchedEvent {
    pub timestamp_ns: u64,

    pub kind: u32,
    pub cpu: u32,

    pub tid: u32,
    pub related_tid: u32,

    pub state: u32,
    pub reserved: u32,
}
