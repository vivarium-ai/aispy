use aya_ebpf::{
    helpers::{bpf_get_current_pid_tgid, bpf_get_smp_processor_id, bpf_ktime_get_ns},
    macros::{btf_tracepoint, map},
    maps::RingBuf,
    programs::BtfTracePointContext,
};

use crate::{
    event::{SchedEvent, SchedEventKind},
    vmlinux::task_struct,
};

#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

#[btf_tracepoint(function = "sched_wakeup")]
pub fn sched_wakeup(ctx: BtfTracePointContext) -> u32 {
    match unsafe { try_sched_wakeup(ctx) } {
        Ok(()) => 0,
        Err(error) => error as u32,
    }
}

unsafe fn try_sched_wakeup(ctx: BtfTracePointContext) -> Result<(), i32> {
    let task: *const task_struct = ctx.arg(0);

    if task.is_null() {
        return Ok(());
    }

    let wakee_tid = unsafe { (*task).pid as u32 };

    let pid_tgid = bpf_get_current_pid_tgid();
    let waker_tid = pid_tgid as u32;

    emit(SchedEvent {
        timestamp_ns: unsafe { bpf_ktime_get_ns() },

        kind: SchedEventKind::Wakeup as u32,
        cpu: unsafe { bpf_get_smp_processor_id() },

        tid: wakee_tid,
        related_tid: waker_tid,

        state: 0,
        reserved: 0,
    });

    Ok(())
}

#[btf_tracepoint(function = "sched_switch")]
pub fn sched_switch(ctx: BtfTracePointContext) -> u32 {
    match unsafe { try_sched_switch(ctx) } {
        Ok(()) => 0,
        Err(error) => error as u32,
    }
}

unsafe fn try_sched_switch(ctx: BtfTracePointContext) -> Result<(), i32> {
    /*
     * Current kernel tracepoint prototype:
     *
     * sched_switch(
     *     bool preempt,
     *     struct task_struct *prev,
     *     struct task_struct *next,
     *     unsigned int prev_state
     * )
     */
    let prev: *const task_struct = ctx.arg(1);
    let next: *const task_struct = ctx.arg(2);
    let prev_state: u32 = ctx.arg(3);

    if prev.is_null() || next.is_null() {
        return Ok(());
    }

    emit(SchedEvent {
        timestamp_ns: unsafe { bpf_ktime_get_ns() },

        kind: SchedEventKind::Switch as u32,
        cpu: unsafe { bpf_get_smp_processor_id() },

        tid: unsafe { (*prev).pid as u32 },
        related_tid: unsafe { (*next).pid as u32 },

        state: prev_state,
        reserved: 0,
    });

    Ok(())
}

fn emit(event: SchedEvent) {
    if let Some(mut entry) = EVENTS.reserve::<SchedEvent>(0) {
        entry.write(event);
        entry.submit(0);
    }
}
