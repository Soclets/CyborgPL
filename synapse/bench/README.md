# Benchmarks

Four small programs written the same way in CyborgPL and C, for CCA against
`clang -O2`. `python3 run.py` builds and runs them (CCA's release build must
exist: `cargo build --release` in `cca/`). Each is built with `--final` when
the final build accepts it, the working build otherwise, and the reason is
printed. Measure with the machine quiet: a game or a video call moves runs
between performance and efficiency cores, and `--ecores` pins both sides to
the same kind when it cannot be quiet.
