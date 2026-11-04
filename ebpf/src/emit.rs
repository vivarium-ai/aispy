use aispy_ebpf_common::WireEvent;
use aya_ebpf::{macros::map, maps::RingBuf};

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(1 << 24, 0);

#[inline(always)]
pub fn emit(event: WireEvent) {
    if let Some(mut entry) = EVENTS.reserve::<WireEvent>(0) {
        entry.write(event);
        entry.submit(0);
    }
}
