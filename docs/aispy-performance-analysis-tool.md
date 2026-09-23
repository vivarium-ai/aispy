# AISpy Performance Analysis Tool

_There is always a well-known solution to every problem — neat, plausible, and wrong._
—H. L. Mencken

_The first principle is that you must not fool yourself and you are the easiest person to fool._
—Richard P. Feynman

## Overview

A modern computer system with hardware accelerator is a complex system of subcomponents (eg CPU, disk, memory, PCIe bus, GPU or other hardware accelerator) that each have highly complex logic. The components cooperate to produce a computation output, but at times also contend or interfere with the ability of a component to operate at maximum thnoughput.

The goal of optimizing the operation of such a computer system is to utilize each necessary component at its maximum performance to the extent that doing so does not reduce the total throughput, which is the maximum amount of work that can be done in a given unit of time.

At times, it is necessary to optimize the performance of a particular subcomponont by reducing the total work necessary to produce a desired result. Other times it is necessary to schedule the operation of a subcomponont such that it is not blocking or preventing another component from performing work efficiently.

A true bottleneck occurs when no available alternate scheduling system can produce an increment of total throughput. At this point, one of the components becomes the limiting factor and a performance optimization, if available, must reduce the duration of the component's work or introduce parallel work at the component to produce an increment of total throughput.

This document describes **AISpy**, a tool that manages the execution of a workload and the collection of all relevant system component metrics such that rigorous data analysis can robustly identify the next available optimization to increase that workload's throughput on a given hardware configuration.

## AISpy Tool

The goal of AISpy is to accurately observe the frog without boiling it or dissecting it. This means that the AI workload being studied, along with the system within which it is executing, are not perturbed or artificial (eg extracting a single algorithm to benchmark in isolation).

To accomplish this, the AISpy approach is composed of three conceptual components:

1. A tool for coordinating and collating performance data;
1. A set of observations from various system components and the workload itself; and
1. A tool for data analysis and possibly visualization of the performance data.

### Architecture of Observations

During the execution of the workload, observations are collected from every component of the system:

1. **Semantic instrumentation:** The semantic information about the code that is executing is derived from source code instrumentation that can run on the host, and via Pliron IR, can be injected into code running on the GPU. This is achieved through a small library that is linked with the workload.
1. **Host execution instrumentation:** Through a small eBPF program and the CPU PMU, host hardware and kernel performance information is collected and written to the data files via the AISpy utility.
1. **GPU execution instrumentation:** Data from CUPTI is collected internal to the workload via the small library.
1. **Resource telemetry:** Data from NVML is also calleced by the AISpy utility external to the workload.


### Orchestration


### Schema

The observation schema contains 11 Parquet tables, which organize conceptually as follows:

1. Provenance:
    1. Runs
    1. Builds
    1. Observation_sites
1. Observations:
    1. Events
    1. Samples
    1. Counters
1. Runtime model:
    1. Entities
    1. Edges
    1. Transfers
    1. Kernel_launches
1. Measurement validity:
    1. Measurement_quality

The goal is sufficient information to support reverse source lookup, causal analysis, hardware attribution, graph queries, and validation.

| Table                         | Purpose                                                                                                                                                                           |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `runs.parquet`                | One row per profiling run: run ID, start/end, workload, machine/software versions, profiler mode, validity state.                                                                 |
| `builds.parquet`              | Exact build provenance: repository, commit, dirty state, rustc version, target triple, profile, compiler flags, build ID.                                                         |
| `observation_sites.parquet`   | Reverse map from `site_id` to Rust definition/source: DefPath-derived identity, crate, function path, source file, line/column, observation kind, build ID.                       |
| `events.parquet`              | Canonical point-in-time observations: semantic begin/end, scheduler events, CUDA API events, module loads, page faults, etc. This is the primary event-sourced record.            |
| `samples.parquet`             | Periodic sampled resource state: CPU utilization/frequency, GPU utilization/clocks/power, PCIe bandwidth, queue depth, etc.                                                       |
| `counters.parquet`            | Counter measurements over a defined interval/range: cycles, instructions, cache misses, HBM bytes, CUPTI metrics, with multiplex/scaling metadata.                                |
| `entities.parquet`            | Runtime entities referenced by observations: tensors, kernels, streams, threads, devices, allocations, etc., with type-specific metadata.                                         |
| `edges.parquet`               | Relationships between entities/operation instances: producer→consumer, dependency, parent/child, synchronization, tensor-flow, control-flow. This is the graph-analysis backbone. |
| `transfers.parquet`           | Explicit data-movement records: H→D, D→H, D→D/P2P, tensor/allocation identity, byte count, memory kind, start/end events.                                                         |
| `kernel_launches.parquet`     | One row per CUDA kernel execution: semantic instance/site, CUPTI correlation ID, kernel entity, stream/device, launch geometry, submit/start/end references.                      |
| `measurement_quality.parquet` | Evidence about the profiler itself: dropped records, sampling periods, PMU multiplex ratios, collector overhead estimates, warnings, collector versions.                          |

#### 1. `runs.parquet`

One row describing the profiling run.

| Column              | Arrow/Parquet type      | Null? | Meaning                                       |
| ------------------- | ----------------------- | ----: | --------------------------------------------- |
| `run_id`            | `FIXED_SIZE_BINARY(16)` |    no | UUIDv7                                        |
| `schema_version`    | `UINT32`                |    no | AiSpy dataset schema                          |
| `state`             | `STRING`                |    no | `complete`, `failed`, `degraded`              |
| `started_at_utc_ns` | `UINT64`                |    no | Wall-clock provenance only                    |
| `ended_at_utc_ns`   | `UINT64`                |   yes | Completion wall time                          |
| `workload_name`     | `STRING`                |   yes | Human-readable workload                       |
| `command`           | `LIST<STRING>`          |    no | Exact argv                                    |
| `profiler_mode`     | `STRING`                |    no | `baseline`, `sampling`, `tracing`, `forensic` |
| `aispy_version`     | `STRING`                |    no | AiSpy version                                 |
| `aispy_build_id`    | `FIXED_SIZE_BINARY(32)` |    no | Exact AiSpy build                             |
| `host_id`           | `STRING`                |    no | Stable machine identifier                     |
| `os`                | `STRING`                |    no | OS/distribution                               |
| `kernel_version`    | `STRING`                |    no | Kernel                                        |
| `cpu_model`         | `STRING`                |   yes | CPU                                           |
| `gpu_summary`       | `STRING`                |   yes | Human-readable GPU summary                    |
| `cuda_version`      | `STRING`                |   yes | CUDA toolkit/runtime                          |
| `driver_version`    | `STRING`                |   yes | NVIDIA driver                                 |
| `notes`             | `STRING`                |   yes | Human annotation                              |

#### 2. `builds.parquet`

Exact software provenance for every instrumented artifact participating in the run.

| Column            | Type                    | Null? | Meaning                                      |
| ----------------- | ----------------------- | ----: | -------------------------------------------- |
| `run_id`          | `FIXED_SIZE_BINARY(16)` |    no | Owning run                                   |
| `build_id`        | `FIXED_SIZE_BINARY(32)` |    no | Content/provenance identity                  |
| `component`       | `STRING`                |    no | `candle`, `attention-rs`, `cuda-oxide`, etc. |
| `repository`      | `STRING`                |   yes | Repository URI                               |
| `git_commit`      | `STRING`                |   yes | Exact commit                                 |
| `git_dirty`       | `BOOLEAN`               |   yes | Working tree differed                        |
| `source_digest`   | `FIXED_SIZE_BINARY(32)` |   yes | Optional exact source-tree digest            |
| `artifact_digest` | `FIXED_SIZE_BINARY(32)` |   yes | Built artifact digest                        |
| `rustc_version`   | `STRING`                |   yes | Compiler                                     |
| `target_triple`   | `STRING`                |   yes | Compilation target                           |
| `profile`         | `STRING`                |   yes | release/debug/custom                         |
| `compiler_flags`  | `LIST<STRING>`          |   yes | Relevant rustc flags                         |
| `cargo_features`  | `LIST<STRING>`          |   yes | Enabled features                             |

#### 3. `observation_sites.parquet`

Reverse mapping from compact runtime identity back to source.

| Column             | Type                    | Null? | Meaning                                     |
| ------------------ | ----------------------- | ----: | ------------------------------------------- |
| `run_id`           | `FIXED_SIZE_BINARY(16)` |    no | Run                                         |
| `site_id`          | `FIXED_SIZE_BINARY(16)` |    no | Stable AiSpy source-site identity           |
| `build_id`         | `FIXED_SIZE_BINARY(32)` |    no | Exact build containing site                 |
| `def_path_hash`    | `FIXED_SIZE_BINARY(16)` |   yes | rustc `DefPathHash` used to derive identity |
| `crate_name`       | `STRING`                |    no | Rust crate                                  |
| `crate_version`    | `STRING`                |   yes | Crate version                               |
| `def_path`         | `STRING`                |    no | e.g. `candle_nn::layer_norm::rms_norm`      |
| `symbol_name`      | `STRING`                |   yes | Emitted/compiler symbol                     |
| `source_file`      | `STRING`                |    no | Repository-relative source path             |
| `line`             | `UINT32`                |   yes | Diagnostic provenance                       |
| `column`           | `UINT32`                |   yes | Diagnostic provenance                       |
| `observation_kind` | `STRING`                |    no | `rmsnorm`, `attention`, etc.                |
| `label`            | `STRING`                |   yes | Optional human description                  |

#### 4. `events.parquet`

The canonical event-sourced record. Something happened at a point in time.

| Column                 | Type                    | Null? | Meaning                                   |
| ---------------------- | ----------------------- | ----: | ----------------------------------------- |
| `run_id`               | `FIXED_SIZE_BINARY(16)` |    no | Run                                       |
| `event_id`             | `UINT64`                |    no | Run-local event ID                        |
| `timestamp`            | `UINT64`                |    no | Timestamp in stated clock domain          |
| `clock_domain`         | `STRING`                |    no | `host_monotonic`, `cupti`, `device`, etc. |
| `source`               | `STRING`                |    no | `aispy`, `perf`, `ebpf`, `cupti`, `nvml`  |
| `kind`                 | `STRING`                |    no | Event type                                |
| `site_id`              | `FIXED_SIZE_BINARY(16)` |   yes | Source observation site                   |
| `instance_id`          | `UINT64`                |   yes | Concrete semantic operation execution     |
| `parent_instance_id`   | `UINT64`                |   yes | Logical parent                            |
| `entity_id`            | `UINT64`                |   yes | Principal affected runtime entity         |
| `thread_entity_id`     | `UINT64`                |   yes | Host thread                               |
| `device_entity_id`     | `UINT64`                |   yes | Device                                    |
| `stream_entity_id`     | `UINT64`                |   yes | CUDA stream                               |
| `cupti_correlation_id` | `UINT64`                |   yes | CUPTI causal join                         |
| `value_i64`            | `INT64`                 |   yes | Optional integer payload                  |
| `value_f64`            | `DOUBLE`                |   yes | Optional numeric payload                  |
| `value_text`           | `STRING`                |   yes | Optional textual payload                  |
| `attributes`           | `MAP<STRING,STRING>`    |   yes | Rare extension metadata                   |

Typical `kind`s:

```text
uop.begin
uop.end
uop.ready
uop.submit

scheduler.wakeup
scheduler.running
scheduler.blocked

cuda.api.enter
cuda.api.exit
cuda.module.load

gpu.kernel.start
gpu.kernel.end

memory.alloc
memory.free

page_fault
```

A span is derived from correlated events.

#### 5. `samples.parquet`

Periodic or statistical observations of resource state.

| Column               | Type                    | Null? | Meaning                     |
| -------------------- | ----------------------- | ----: | --------------------------- |
| `run_id`             | `FIXED_SIZE_BINARY(16)` |    no | Run                         |
| `sample_id`          | `UINT64`                |    no | Run-local ID                |
| `timestamp`          | `UINT64`                |    no | Sample time                 |
| `clock_domain`       | `STRING`                |    no | Timestamp domain            |
| `source`             | `STRING`                |    no | Collector                   |
| `resource_entity_id` | `UINT64`                |    no | CPU/GPU/bus/etc.            |
| `metric`             | `STRING`                |    no | Metric name                 |
| `value`              | `DOUBLE`                |    no | Measurement                 |
| `unit`               | `STRING`                |    no | `%`, `Hz`, `B/s`, `W`, etc. |
| `sampling_period_ns` | `UINT64`                |   yes | Expected sampling interval  |
| `attributes`         | `MAP<STRING,STRING>`    |   yes | Source-specific qualifiers  |

Examples:

```text
cpu.utilization
cpu.frequency
gpu.sm_active
gpu.power
gpu.clock
pcie.rx_bytes_per_sec
hbm.read_bytes_per_sec
```

#### 6. `counters.parquet`

Hardware/software counters collected across a defined measurement range.

| Column               | Type                    | Null? | Meaning                            |
| -------------------- | ----------------------- | ----: | ---------------------------------- |
| `run_id`             | `FIXED_SIZE_BINARY(16)` |    no | Run                                |
| `measurement_id`     | `UINT64`                |    no | Counter observation                |
| `source`             | `STRING`                |    no | `perf`, `cupti`, etc.              |
| `resource_entity_id` | `UINT64`                |    no | Resource measured                  |
| `site_id`            | `FIXED_SIZE_BINARY(16)` |   yes | Semantic site if attributable      |
| `instance_id`        | `UINT64`                |   yes | Specific execution if attributable |
| `start_event_id`     | `UINT64`                |   yes | Beginning of measurement range     |
| `end_event_id`       | `UINT64`                |   yes | End                                |
| `metric`             | `STRING`                |    no | `cycles`, `instructions`, etc.     |
| `raw_value`          | `DOUBLE`                |    no | Direct collector result            |
| `scaled_value`       | `DOUBLE`                |   yes | Corrected/multiplex-adjusted value |
| `unit`               | `STRING`                |    no | Metric unit                        |
| `time_enabled_ns`    | `UINT64`                |   yes | PMU availability interval          |
| `time_running_ns`    | `UINT64`                |   yes | Actual PMU counting interval       |
| `attributes`         | `MAP<STRING,STRING>`    |   yes | Metric qualifiers                  |

#### 7. `entities.parquet`

Runtime objects referenced by observations.

| Column                 | Type                    | Null? | Meaning                                                                  |
| ---------------------- | ----------------------- | ----: | ------------------------------------------------------------------------ |
| `run_id`               | `FIXED_SIZE_BINARY(16)` |    no | Run                                                                      |
| `entity_id`            | `UINT64`                |    no | Run-local identity                                                       |
| `entity_type`          | `STRING`                |    no | `tensor`, `kernel`, `thread`, `gpu`, `cpu`, `stream`, `allocation`, etc. |
| `parent_entity_id`     | `UINT64`                |   yes | Structural parent                                                        |
| `name`                 | `STRING`                |   yes | Human-readable name                                                      |
| `site_id`              | `FIXED_SIZE_BINARY(16)` |   yes | Defining source site                                                     |
| `created_event_id`     | `UINT64`                |   yes | Lifetime beginning                                                       |
| `destroyed_event_id`   | `UINT64`                |   yes | Lifetime end                                                             |
| `size_bytes`           | `UINT64`                |   yes | Tensor/allocation size                                                   |
| `dtype`                | `STRING`                |   yes | Tensor datatype                                                          |
| `shape`                | `LIST<INT64>`           |   yes | Tensor shape                                                             |
| `memory_space`         | `STRING`                |   yes | host/device/unified/shared/etc.                                          |
| `symbol`               | `STRING`                |   yes | Kernel/symbol identity                                                   |
| `module`               | `STRING`                |   yes | CUDA/PTX/module                                                          |
| `registers_per_thread` | `UINT32`                |   yes | Kernel metadata                                                          |
| `static_shared_bytes`  | `UINT64`                |   yes | Kernel metadata                                                          |
| `process_id`           | `UINT32`                |   yes | Thread/process entity                                                    |
| `thread_id`            | `UINT64`                |   yes | Host thread                                                              |
| `device_ordinal`       | `UINT32`                |   yes | GPU/CPU device index                                                     |
| `attributes`           | `MAP<STRING,STRING>`    |   yes | Uncommon type-specific properties                                        |

#### 8. `edges.parquet`

Semantic/dataflow relationships between operation **instances**.

| Column               | Type                    | Null? | Meaning                                          |
| -------------------- | ----------------------- | ----: | ------------------------------------------------ |
| `run_id`             | `FIXED_SIZE_BINARY(16)` |    no | Run                                              |
| `edge_id`            | `UINT64`                |    no | Edge                                             |
| `src_instance_id`    | `UINT64`                |    no | Producer/predecessor                             |
| `dst_instance_id`    | `UINT64`                |    no | Consumer/successor                               |
| `edge_kind`          | `STRING`                |    no | `data`, `control`, `dependency`, `sync`, `queue` |
| `entity_id`          | `UINT64`                |   yes | Tensor/allocation carrying dependency            |
| `created_event_id`   | `UINT64`                |   yes | Event establishing dependency                    |
| `satisfied_event_id` | `UINT64`                |   yes | Event satisfying dependency                      |
| `bytes`              | `UINT64`                |   yes | Data volume represented by edge                  |
| `attributes`         | `MAP<STRING,STRING>`    |   yes | Exceptional metadata                             |

This is the principal DuckDB/DuckPGQ graph:

```text
operation instance
       │
       ├── tensor/data dependency
       ▼
operation instance
```

#### 9. `transfers.parquet`

A structured description of a transfer. Timing remains anchored to canonical events.

| Column                 | Type                    | Null? | Meaning                             |
| ---------------------- | ----------------------- | ----: | ----------------------------------- |
| `run_id`               | `FIXED_SIZE_BINARY(16)` |    no | Run                                 |
| `transfer_id`          | `UINT64`                |    no | Transfer                            |
| `instance_id`          | `UINT64`                |   yes | Semantic operation responsible      |
| `site_id`              | `FIXED_SIZE_BINARY(16)` |   yes | Source site                         |
| `tensor_entity_id`     | `UINT64`                |   yes | Tensor being moved                  |
| `src_entity_id`        | `UINT64`                |    no | Source memory/device                |
| `dst_entity_id`        | `UINT64`                |    no | Destination                         |
| `size_bytes`           | `UINT64`                |    no | Transfer size                       |
| `memory_kind`          | `STRING`                |   yes | pageable/pinned/device/unified/etc. |
| `submit_event_id`      | `UINT64`                |   yes | Host submission                     |
| `start_event_id`       | `UINT64`                |    no | Actual transfer start               |
| `end_event_id`         | `UINT64`                |    no | Completion                          |
| `stream_entity_id`     | `UINT64`                |   yes | CUDA stream                         |
| `cupti_correlation_id` | `UINT64`                |   yes | CUDA causal linkage                 |
| `is_async`             | `BOOLEAN`               |   yes | Async transfer request              |
| `attributes`           | `MAP<STRING,STRING>`    |   yes | Additional transfer metadata        |

Derived:

```text
duration
effective bandwidth
H→D / D→H totals
transfer-size distribution
overlap with compute
```
#### 10. `kernel_launches.parquet`

Structured launch identity and provenance. Again, timestamps remain events.

| Column                 | Type                    | Null? | Meaning                            |
| ---------------------- | ----------------------- | ----: | ---------------------------------- |
| `run_id`               | `FIXED_SIZE_BINARY(16)` |    no | Run                                |
| `launch_id`            | `UINT64`                |    no | Launch                             |
| `instance_id`          | `UINT64`                |   yes | Semantic operation execution       |
| `site_id`              | `FIXED_SIZE_BINARY(16)` |   yes | Launch/source site                 |
| `kernel_entity_id`     | `UINT64`                |    no | Kernel in `entities`               |
| `thread_entity_id`     | `UINT64`                |   yes | Launching CPU thread               |
| `device_entity_id`     | `UINT64`                |    no | GPU                                |
| `stream_entity_id`     | `UINT64`                |    no | CUDA stream                        |
| `cupti_correlation_id` | `UINT64`                |   yes | CUPTI launch/activity relationship |
| `api_enter_event_id`   | `UINT64`                |   yes | CUDA API entered                   |
| `api_exit_event_id`    | `UINT64`                |   yes | CUDA API returned                  |
| `gpu_start_event_id`   | `UINT64`                |    no | Kernel actually started            |
| `gpu_end_event_id`     | `UINT64`                |    no | Kernel completed                   |
| `grid_x`               | `UINT32`                |    no | Grid dimensions                    |
| `grid_y`               | `UINT32`                |    no |                                    |
| `grid_z`               | `UINT32`                |    no |                                    |
| `block_x`              | `UINT32`                |    no | Block dimensions                   |
| `block_y`              | `UINT32`                |    no |                                    |
| `block_z`              | `UINT32`                |    no |                                    |
| `dynamic_shared_bytes` | `UINT64`                |   yes | Dynamic shared memory              |
| `attributes`           | `MAP<STRING,STRING>`    |   yes | Launch-specific extension          |

This gives us:

```text
Rust site
   ↓
semantic execution
   ↓
CUDA API call
   ↓
CUPTI correlation
   ↓
specific kernel
   ↓
actual GPU execution
```

#### 11. `measurement_quality.parquet`

The profiler observes itself.

| Column                  | Type                    | Null? | Meaning                             |
| ----------------------- | ----------------------- | ----: | ----------------------------------- |
| `run_id`                | `FIXED_SIZE_BINARY(16)` |    no | Run                                 |
| `quality_id`            | `UINT64`                |    no | Observation                         |
| `collector`             | `STRING`                |    no | `perf`, `cupti`, `ebpf`, etc.       |
| `collector_version`     | `STRING`                |   yes | Implementation/API version          |
| `start_event_id`        | `UINT64`                |   yes | Quality interval start              |
| `end_event_id`          | `UINT64`                |   yes | Quality interval end                |
| `records_seen`          | `UINT64`                |   yes | Total observed                      |
| `records_dropped`       | `UINT64`                |   yes | Known losses                        |
| `sampling_period_ns`    | `UINT64`                |   yes | Requested sampling rate             |
| `multiplex_ratio`       | `DOUBLE`                |   yes | PMU time-running/time-enabled       |
| `overhead_estimate_pct` | `DOUBLE`                |   yes | Estimated measurement perturbation  |
| `status`                | `STRING`                |    no | `valid`, `degraded`, `invalid`      |
| `warning`               | `STRING`                |   yes | Human-readable reason               |
| `attributes`            | `MAP<STRING,STRING>`    |   yes | Collector-specific quality evidence |

### Analysis

Once data is collected from a workload run, it is available for analysis. 

### Validation

The foundation for robust data depends on three attributes:

1. **Valid:** The data is actually measuring the correct thing.
1. **Variable:** The data has a wide enough variability to capture the full range of the values being measured.
1. **Reliable:** The data does not "wiggle" on its own.

## CLI Reference

TK

## Glossary

| Component                      | Role                                                                                                                                                                                                                                     |
| ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Workload**                   | The program being measured: typically Candle, `attention.rs`, or another Rust inference runtime. It produces useful work and semantic operations such as RMSNorm, attention, KV-cache access, and tensor movement.                       |
| **Semantic observation site**  | A source-level point representing meaningful work. It is identified automatically from compiler information such as Rust's `DefPathHash`; developers do not assign numeric IDs manually.                                                 |
| **Observation instance**       | One concrete execution of an observation site during a run. Many instances may correspond to the same source site.                                                                                                                       |
| **AiSpy annotation**           | A minimal Rust attribute used where automatic discovery cannot provide enough semantic meaning, e.g. `#[observe(kind = "rmsnorm")]`. It describes intent; it does not perform measurement itself.                                        |
| **rustc**                      | The Rust compiler. It provides resolved program identity and source provenance, including definition paths and source spans, allowing observations to map reliably back to source code.                                                  |
| **cuda-oxide**                 | Rust-to-CUDA compiler/runtime machinery. It gives AiSpy access to the path from Rust device code through compiler IR to generated GPU kernels.                                                                                           |
| **Pliron IR**                  | cuda-oxide's intermediate representation infrastructure. AiSpy can carry semantic identities and provenance through it and, where necessary, insert device-side measurement operations during compilation.                               |
| **CPU**                        | Executes host code, scheduling, runtime logic, tensor preparation, CUDA calls, and other host-side computation.                                                                                                                          |
| **Host memory**                | DRAM used by the CPU and as a source/destination for device transfers. Its bandwidth, latency, paging, NUMA placement, and contention can determine apparent program performance.                                                        |
| **Storage**                    | Disk or other persistent storage feeding model/data state. It matters when I/O or paging enters the critical path.                                                                                                                       |
| **PCIe / interconnect**        | The communication path between host and GPU, or between GPUs. Transfer volume, transaction size, latency, bandwidth, and overlap with compute are first-class performance concerns.                                                      |
| **GPU**                        | Executes device kernels and maintains device-local state. Relevant resources include SMs, registers, caches, shared memory, HBM, copy engines, and execution queues.                                                                     |
| **CUDA runtime/driver**        | The host-side NVIDIA software layer that launches kernels, moves memory, synchronizes streams, loads modules, and manages device resources.                                                                                              |
| **CUPTI**                      | NVIDIA's authoritative profiling interface. It reports CUDA API activity, kernel execution, copies, counters, PC samples, and NVIDIA-provided correlation IDs linking host CUDA calls to resulting GPU work.                             |
| **Linux `perf` / PMU**         | Provides hardware CPU measurements such as cycles, instructions, cache misses, branches, and other processor counters.                                                                                                                   |
| **eBPF / Linux tracepoints**   | Exposes operating-system behavior that application timing cannot explain: scheduling, wakeups, blocking, page faults, block I/O, and related kernel activity.                                                                            |
| **NVML**                       | NVIDIA's management/telemetry API. It supplies coarse GPU state such as utilization, clocks, power, temperature, and memory usage.                                                                                                       |
| **Host monotonic clock**       | AiSpy's authoritative timebase for events it records directly on the host. Wall-clock time is retained only for provenance.                                                                                                              |
| **CUPTI activity time**        | NVIDIA-provided timing for actual GPU activity. AiSpy uses CUPTI's timing rather than attempting to synchronize a hand-built CPU clock with raw GPU timers.                                                                              |
| **CUPTI correlation ID**       | NVIDIA-generated causal identity linking a CUDA host API invocation to its corresponding kernel, copy, or other GPU activity. It is an identifier, not a timestamp.                                                                      |
| **AiSpy**                      | The Rust executable coordinating the entire measurement run: configuration, run identity, collector startup/shutdown, observation ingestion, normalization, quality accounting, Parquet output, verification, and analysis entry points. |
| **Collector**                  | An AiSpy adapter for one authoritative measurement source, such as `perf`, CUPTI, NVML, or eBPF. Collectors report observations through one common interface and do not independently own storage or identity.                           |
| **Event**                      | The fundamental stored observation: something happened at a particular time on a particular resource. Begin/end events may later be paired into intervals, but events remain the canonical evidence.                                     |
| **Sample**                     | A point-in-time measurement of resource state, such as GPU utilization, CPU frequency, power, or PCIe throughput.                                                                                                                        |
| **Counter measurement**        | A quantity accumulated over a defined measurement range, such as CPU cycles, cache misses, HBM traffic, or GPU hardware counters.                                                                                                        |
| **Entity**                     | A runtime object referenced by observations: tensor, kernel, allocation, thread, stream, CPU, GPU, etc.                                                                                                                                  |
| **Edge**                       | A relationship between operation instances, such as producer→consumer, data dependency, synchronization, or control dependency. These form the execution/dataflow graph.                                                                 |
| **Run ID**                     | A UUIDv7 identifying one measurement run. It provides globally unique, sortable identity without hand-maintained numbering.                                                                                                              |
| **Build ID**                   | A cryptographic identity for the exact software artifact/source provenance used in a run. It connects measurements to repository revision, compiler configuration, and binaries.                                                         |
| **Arrow**                      | The in-memory columnar representation used to batch normalized AiSpy records efficiently before persistence.                                                                                                                             |
| **Parquet**                    | The canonical immutable storage format for a run's observations and provenance. It preserves typed, columnar data efficiently and independently of any analysis engine.                                                                  |
| **DuckDB**                     | The primary analysis engine. It queries Parquet directly to correlate observations, calculate derived intervals and attribution, traverse dependency graphs, compare runs, and identify bottlenecks.                                     |
| **Perfetto**                   | Optional visualization tooling for human inspection of complex timelines. It is not canonical storage and is not required for AiSpy analysis.                                                                                            |
| **Measurement-quality record** | Evidence about the profiler itself: dropped records, sampling periods, PMU multiplexing, collector errors, and estimated observer overhead. It determines whether a result is trustworthy.                                               |
| **Run manifest and seal**      | The immutable description of a completed run: configuration, environment, artifacts, file hashes, and optionally a digital signature. It establishes integrity and provenance independently of performance conclusions.                  |

## References

1. [A Holistic White-Box Approach to Performance Modeling for Supercomputing](https://open.fau.de/handle/openfau/40254)
1. [Budgeting Bytes: A Windowed Storage Roofline and Dual-Budget Architecture Ablations for Storage-Bound LLM Decoding](https://arxiv.org/abs/2609.04238)
1. [Hierarchical Roofline Analysis: How to Collect Data using Performance Tools on Intel CPUs and NVIDIA GPUs](https://arxiv.org/abs/2009.02449)
1. [User Guide — Nsight Systems](https://docs.nvidia.com/nsight-systems/UserGuide/index.html)
