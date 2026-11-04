#![no_std]
#![no_main]

mod emit;
mod sched;
mod vmlinux;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
