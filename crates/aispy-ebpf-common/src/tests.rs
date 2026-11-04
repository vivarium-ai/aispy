use core::mem::{align_of, offset_of, size_of};

use crate::*;

#[test]
fn event_kind_has_expected_representation() {
    assert_eq!(size_of::<EventKind>(), 2);
    assert_eq!(align_of::<EventKind>(), 2);
}

#[test]
fn wire_event_has_expected_layout() {
    assert_eq!(size_of::<WireEvent>(), 48);
    assert_eq!(align_of::<WireEvent>(), 8);

    assert_eq!(offset_of!(WireEvent, timestamp_ns), 0);
    assert_eq!(offset_of!(WireEvent, cpu), 8);
}

#[test]
fn event_kind_values_are_stable() {
    assert_eq!(EventKind::SchedSwitch as u16, 1);
    assert_eq!(EventKind::SchedWakeup as u16, 2);
    assert_eq!(EventKind::SchedWakeupNew as u16, 3);
    assert_eq!(EventKind::SchedMigrate as u16, 4);

    assert_eq!(EventKind::ProcessFork as u16, 100);
    assert_eq!(EventKind::ProcessExec as u16, 101);
    assert_eq!(EventKind::ProcessExit as u16, 102);

    assert_eq!(EventKind::IrqEntry as u16, 200);
    assert_eq!(EventKind::IrqExit as u16, 201);
    assert_eq!(EventKind::SoftIrqRaise as u16, 202);
    assert_eq!(EventKind::SoftIrqEntry as u16, 203);
    assert_eq!(EventKind::SoftIrqExit as u16, 204);

    assert_eq!(EventKind::WorkqueueQueue as u16, 300);
    assert_eq!(EventKind::WorkqueueStart as u16, 301);
    assert_eq!(EventKind::WorkqueueEnd as u16, 302);

    assert_eq!(EventKind::BlockIssue as u16, 400);
    assert_eq!(EventKind::BlockComplete as u16, 401);

    assert_eq!(EventKind::PageFault as u16, 500);
    assert_eq!(EventKind::ReclaimBegin as u16, 501);
    assert_eq!(EventKind::ReclaimEnd as u16, 502);
    assert_eq!(EventKind::OomKill as u16, 503);

    assert_eq!(EventKind::CpuIdle as u16, 600);
    assert_eq!(EventKind::CpuFrequency as u16, 601);

    assert_eq!(EventKind::SignalGenerate as u16, 700);
    assert_eq!(EventKind::SignalDeliver as u16, 701);

    assert_eq!(EventKind::SyscallEnter as u16, 800);
    assert_eq!(EventKind::SyscallExit as u16, 801);

    assert_eq!(EventKind::NetRx as u16, 900);
    assert_eq!(EventKind::NetTx as u16, 901);
    assert_eq!(EventKind::TcpRetransmit as u16, 902);
}

#[test]
fn sched_switch_round_trips() {
    let payload = SchedSwitch {
        prev_tid: 101,
        next_tid: 202,
        prev_state: 7,
    };

    let event = WireEvent::sched_switch(1_000, 3, payload);

    assert_eq!(event.timestamp_ns, 1_000);
    assert_eq!(event.cpu, 3);
    assert_eq!(event.kind(), Some(EventKind::SchedSwitch));
    assert_eq!(event.as_sched_switch(), Some(payload));
    assert_eq!(event.as_sched_wakeup(), None);
}

#[test]
fn sched_wakeup_round_trips() {
    let payload = SchedWakeup {
        waker_tid: 101,
        wakee_tid: 202,
        target_cpu: 7,
    };

    let event = WireEvent::sched_wakeup(1_000, 3, payload);

    assert_eq!(event.kind(), Some(EventKind::SchedWakeup));
    assert_eq!(event.as_sched_wakeup(), Some(payload));
}

#[test]
fn sched_wakeup_new_round_trips() {
    let payload = SchedWakeup {
        waker_tid: 101,
        wakee_tid: 202,
        target_cpu: 7,
    };

    let event = WireEvent::sched_wakeup_new(1_000, 3, payload);

    assert_eq!(event.kind(), Some(EventKind::SchedWakeupNew));
    assert_eq!(event.as_sched_wakeup(), Some(payload));
}

#[test]
fn sched_migrate_round_trips() {
    let payload = SchedMigrate {
        tid: 101,
        destination_cpu: 8,
    };

    let event = WireEvent::sched_migrate(1_000, 3, payload);

    assert_eq!(event.as_sched_migrate(), Some(payload));
}

#[test]
fn process_events_round_trip() {
    let fork = ProcessFork {
        parent_tid: 10,
        parent_tgid: 10,
        child_tid: 20,
        child_tgid: 20,
    };

    let event = WireEvent::process_fork(1_000, 0, fork);
    assert_eq!(event.as_process_fork(), Some(fork));

    let exec = ProcessExec { tid: 20, tgid: 20 };

    let event = WireEvent::process_exec(2_000, 0, exec);
    assert_eq!(event.as_process_exec(), Some(exec));

    let exit = ProcessExit {
        tid: 20,
        tgid: 20,
        exit_code: -9,
    };

    let event = WireEvent::process_exit(3_000, 0, exit);
    assert_eq!(event.as_process_exit(), Some(exit));
}

#[test]
fn interrupt_events_round_trip() {
    let irq = Irq { irq: 42 };

    let event = WireEvent::irq_entry(1_000, 2, irq);
    assert_eq!(event.as_irq(), Some(irq));

    let soft_irq = SoftIrq { vector: 6 };

    let event = WireEvent::soft_irq_entry(1_000, 2, soft_irq);

    assert_eq!(event.as_soft_irq(), Some(soft_irq));
}

#[test]
fn workqueue_round_trips() {
    let payload = Workqueue {
        work_id: 0x1234_5678,
        function: 0xabcd_ef01,
    };

    let event = WireEvent::workqueue_start(1_000, 2, payload);

    assert_eq!(event.as_workqueue(), Some(payload));
}

#[test]
fn block_io_round_trips() {
    let payload = BlockIo {
        request_id: 0x1234_5678,
        device: 0x9876,
        sector: 123_456,
        bytes: 65_536,
        operation: 3,
    };

    let event = WireEvent::block_issue(1_000, 2, payload);

    assert_eq!(event.as_block_io(), Some(payload));
}

#[test]
fn memory_events_round_trip() {
    let fault = PageFault {
        address: 0x1234_5678_9abc_def0,
        tid: 101,
        tgid: 100,
        flags: 0x55aa,
    };

    let event = WireEvent::page_fault(1_000, 2, fault);
    assert_eq!(event.as_page_fault(), Some(fault));

    let reclaim = Reclaim {
        pages: 512,
        tid: 101,
        tgid: 100,
    };

    let event = WireEvent::reclaim_end(1_000, 2, reclaim);

    assert_eq!(event.as_reclaim(), Some(reclaim));

    let oom = OomKill {
        victim_tid: 202,
        victim_tgid: 200,
    };

    let event = WireEvent::oom_kill(1_000, 2, oom);
    assert_eq!(event.as_oom_kill(), Some(oom));
}

#[test]
fn cpu_events_round_trip() {
    let idle = CpuIdle { state: 4 };

    let event = WireEvent::cpu_idle(1_000, 2, idle);
    assert_eq!(event.as_cpu_idle(), Some(idle));

    let frequency = CpuFrequency {
        frequency_khz: 3_200_000,
    };

    let event = WireEvent::cpu_frequency(1_000, 2, frequency);
    assert_eq!(event.as_cpu_frequency(), Some(frequency));
}

#[test]
fn signal_round_trips() {
    let payload = Signal {
        signal: 15,
        source_tid: 101,
        source_tgid: 100,
        target_tid: 202,
        target_tgid: 200,
    };

    let event = WireEvent::signal_generate(1_000, 2, payload);

    assert_eq!(event.as_signal(), Some(payload));
}

#[test]
fn syscall_events_round_trip() {
    let enter = SyscallEnter {
        tid: 101,
        tgid: 100,
        syscall: 257,
    };

    let event = WireEvent::syscall_enter(1_000, 2, enter);
    assert_eq!(event.as_syscall_enter(), Some(enter));

    let exit = SyscallExit {
        tid: 101,
        tgid: 100,
        syscall: 257,
        result: -2,
    };

    let event = WireEvent::syscall_exit(2_000, 2, exit);
    assert_eq!(event.as_syscall_exit(), Some(exit));
}

#[test]
fn network_events_round_trip() {
    let network = Network {
        socket_id: 0x1234_5678,
        tid: 101,
        tgid: 100,
        bytes: 4096,
    };

    let event = WireEvent::net_rx(1_000, 2, network);

    assert_eq!(event.as_network(), Some(network));

    let retransmit = TcpRetransmit {
        socket_id: 0x1234_5678,
        sequence: 0xdead_beef,
    };

    let event = WireEvent::tcp_retransmit(2_000, 2, retransmit);

    assert_eq!(event.as_tcp_retransmit(), Some(retransmit));
}

#[test]
fn accessors_reject_wrong_event_kinds() {
    let event = WireEvent::sched_switch(
        1_000,
        2,
        SchedSwitch {
            prev_tid: 101,
            next_tid: 202,
            prev_state: 0,
        },
    );

    assert!(event.as_sched_switch().is_some());

    assert!(event.as_sched_wakeup().is_none());
    assert!(event.as_sched_migrate().is_none());
    assert!(event.as_process_fork().is_none());
    assert!(event.as_irq().is_none());
    assert!(event.as_block_io().is_none());
    assert!(event.as_page_fault().is_none());
    assert!(event.as_syscall_enter().is_none());
    assert!(event.as_network().is_none());
}
