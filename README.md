# AISpy Performance Analysis Tool

AISpy (eye-spy) is a Rust-native performance instrumentation and analysis tool for heterogeneous CPU/GPU workloads.

It correlates semantic application events with CPU/OS execution data, CUDA/GPU activity, and system telemetry, then stores the resulting observations in Parquet for analysis with DuckDB.

## Build

```sh
cargo build --release --features cli
```

To build only the lightweight Rust library:

```sh
cargo build --lib --no-default-features
```

## Test

```sh
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Contributing

Contributions are welcome.  Please:

1. format changes with `cargo fmt`;
2. run `cargo clippy --all-targets --all-features -- -D warnings`;
3. run the full test suite;
4. keep instrumentation overhead and measurement validity in mind;

Open an issue before making substantial architectural changes.

## License

Licensed under the MIT license.
