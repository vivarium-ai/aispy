#[cfg(not(target_os = "linux"))]
compile_error!(
    "AiSpy is Linux-only. It depends on Linux performance interfaces such as \
     perf_event_open, eBPF, and kernel tracepoints."
);

extern crate self as aispy;

pub mod entities;
pub mod event;
pub mod observation;

pub use aispy_macros::observe;
