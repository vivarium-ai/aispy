use aispy_ebpf_common::{SchedMigrate, SchedSwitch, SchedWakeup, UNKNOWN_CPU, WireEvent};
use aya_ebpf::{
    helpers::{bpf_get_current_pid_tgid, bpf_get_smp_processor_id, bpf_ktime_get_ns},
    macros::btf_tracepoint,
    programs::BtfTracePointContext,
};

use crate::{emit::emit, vmlinux::task_struct};

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
    let waker_tid = bpf_get_current_pid_tgid() as u32;

    let event = WireEvent::sched_wakeup(
        unsafe { bpf_ktime_get_ns() },
        unsafe { bpf_get_smp_processor_id() },
        SchedWakeup {
            waker_tid,
            wakee_tid,
            target_cpu: UNKNOWN_CPU,
        },
    );

    emit(event);

    Ok(())
}

#[btf_tracepoint(function = "sched_wakeup_new")]
pub fn sched_wakeup_new(ctx: BtfTracePointContext) -> u32 {
    match unsafe { try_sched_wakeup_new(ctx) } {
        Ok(()) => 0,
        Err(error) => error as u32,
    }
}

unsafe fn try_sched_wakeup_new(ctx: BtfTracePointContext) -> Result<(), i32> {
    let task: *const task_struct = ctx.arg(0);

    if task.is_null() {
        return Ok(());
    }

    let wakee_tid = unsafe { (*task).pid as u32 };
    let waker_tid = bpf_get_current_pid_tgid() as u32;

    let event = WireEvent::sched_wakeup_new(
        unsafe { bpf_ktime_get_ns() },
        unsafe { bpf_get_smp_processor_id() },
        SchedWakeup {
            waker_tid,
            wakee_tid,
            target_cpu: u32::MAX,
        },
    );

    emit(event);

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

    let event = WireEvent::sched_switch(
        unsafe { bpf_ktime_get_ns() },
        unsafe { bpf_get_smp_processor_id() },
        SchedSwitch {
            prev_tid: unsafe { (*prev).pid as u32 },
            next_tid: unsafe { (*next).pid as u32 },
            prev_state,
        },
    );

    emit(event);

    Ok(())
}

#[btf_tracepoint(function = "sched_migrate_task")]
pub fn sched_migrate_task(ctx: BtfTracePointContext) -> u32 {
    match unsafe { try_sched_migrate_task(ctx) } {
        Ok(()) => 0,
        Err(error) => error as u32,
    }
}

unsafe fn try_sched_migrate_task(ctx: BtfTracePointContext) -> Result<(), i32> {
    let task: *const task_struct = ctx.arg(0);
    let destination_cpu: u32 = ctx.arg(1);

    if task.is_null() {
        return Ok(());
    }

    let event = WireEvent::sched_migrate(
        unsafe { bpf_ktime_get_ns() },
        unsafe { bpf_get_smp_processor_id() },
        SchedMigrate {
            tid: unsafe { (*task).pid as u32 },
            destination_cpu,
        },
    );

    emit(event);

    Ok(())
}
