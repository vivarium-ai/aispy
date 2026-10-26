#![no_std]
#![no_main]

mod event;
mod sched;

#[allow(
    clippy::all,
    dead_code,
    improper_ctypes_definitions,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unnecessary_transmutes,
    unsafe_op_in_unsafe_fn,
)]
#[rustfmt::skip]
mod vmlinux;

#[cfg(not(test))]
#[panic_handler]
fn panic(
    _info: &core::panic::PanicInfo,
) -> ! {
    loop {}
}
