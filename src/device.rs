#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEventKind {
    Marker = 1,
    RangeBegin = 2,
    RangeEnd = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceEvent {
    pub timestamp: u64,

    pub site_id_hi: u64,
    pub site_id_lo: u64,

    pub instance_id: u64,

    pub kind: u32,
    pub device_id: u32,

    pub block_x: u32,
    pub block_y: u32,
    pub block_z: u32,

    pub warp_id: u32,

    pub payload: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceSiteId {
    pub hi: u64,
    pub lo: u64,
}

impl DeviceSiteId {
    pub const fn from_parts(hi: u64, lo: u64) -> Self {
        Self { hi, lo }
    }
}
