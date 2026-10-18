# Changelog

All notable changes to **SEED-Core** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.4] - 2026-10-18

### Added
- **Formal Threat Model (`docs/THREAT_MODEL.md`)**: Comprehensive breakdown of security invariants, in-scope vs. out-of-scope attack vectors, and host sovereignty defenses.
- **SEED-Core vs. LLM-as-a-Judge Study (`docs/BENCHMARK_VS_LLM.md`)**: In-depth empirical latency, cost, and jailbreak vulnerability comparison.
- **CI Artifact Publishing**: Automated upload of `bench/results/latest.json` on every CI run via GitHub Actions.
- **Supply Chain Security**: Integrated `cargo audit` into GitHub Actions workflow scanning for dependency vulnerabilities.
- **Known Limitations Section**: Transparent boundary definitions in `README.md` (domain LID calibration, OS sandbox distinction).

### Changed
- Benchmark runner `bench/run_benchmarks.sh` now outputs structured machine-readable JSON to `bench/results/latest.json`.
- Updated README hero badges with CI status, Crates.io, and dual licensing.

---

## [0.1.3] - 2026-10-18

### Added
- **Strategic Value Proposition**: Positioned SEED-Core as a deterministic, sub-15µs replacement for paid LLM deciders (cutting costs by 90%+).
- **Automated CI Workflow (`.github/workflows/ci.yml`)**: Continuous integration testing for Rust (`cargo test`, `clippy`, `fmt`) and Python SDK (matrix across Python 3.10–3.12).
- **Public Reproducibility Benchmark Suite (`bench/run_benchmarks.sh`)**: Script executing 10,000 iterations measuring median latency (14.4 µs) and throughput (65,000+ decisions/sec).
- **Migration Guide & Demo (`examples/migrate_from_jev.py`)**: End-to-end runnable demonstration of migrating from LLM-as-a-judge to SEED-Core.
- **Security & Contribution Standards**: Added `SECURITY.md` (Responsible Disclosure Policy) and `CONTRIBUTING.md`.

### Fixed
- Clippy code quality warnings resolved with zero warnings under `-D warnings`.
- C-FFI safety contracts documented with explicit `# Safety` invariants.

---

## [0.1.2] - 2026-10-17

### Changed
- Release version bump for crates.io registry synchronization.

---

## [0.1.1] - 2026-10-17

### Added
- **Official Python SDK (`seed-core`)**: Native C-FFI wrapper delivering ~15 µs evaluation directly in Python.
- **Python Wheel Distribution**: Automated wheel packaging in `python/dist/`.
- **`HOST_SOVEREIGN` Profile**: Unshackled host execution & browser automation with strict hardware-native guardrails.
- **Browser Security Rules**: Verification suite intercepting SSRF and loopback exfiltration vectors.

---

## [0.1.0] - 2026-10-17

### Initial Release
- Sub-microsecond deterministic 2-stage decision engine in native Rust.
- Real-time episodic memory store (`KnowledgeStore`) with Local Intrinsic Dimensionality (LID) estimation.
- Cryptographic SHA-256 audit proof chain.
- Asynchronous Unix Domain Socket and TCP IPC server (`tokio`).
- Initial 11/11 Rust unit and policy tests passing.
- Dual Community and Commercial licensing structure.
