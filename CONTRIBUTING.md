# Contributing to SEED-Core

We welcome community contributions, architectural improvements, policy rule extensions, and language SDK bindings to **SEED-Core**!

---

## Code of Conduct

SEED-Core is an open, respectful, and high-standard engineering environment. We value empirical evidence, reproducible benchmarks, and deterministic safety over speculation.

---

## Development Environment Setup

### Prerequisites
- **Rust Toolchain:** Stable Rust (2021 edition), `clippy`, and `rustfmt`.
  ```bash
  rustup update stable
  rustup component add clippy rustfmt
  ```
- **Python:** Python 3.10+ with `pytest` and `setuptools`.

### Building and Testing
Before submitting any Pull Request:

1. **Format Code:**
   ```bash
   cargo fmt --all -- --check
   ```
2. **Run Linter:**
   ```bash
   cargo clippy --all-targets --all-features -- -D warnings
   ```
3. **Run Native Tests:**
   ```bash
   cargo test
   ```
4. **Run Python SDK Verification:**
   ```bash
   PYTHONPATH=python python3 python/test_python_sdk.py
   ```
5. **Run the Full Benchmark Suite:**
   ```bash
   ./bench/run_benchmarks.sh
   ```

---

## Architectural Principles

When writing or extending constitutional rules in `src/decider.rs`:
1. **Deterministic Latency Invariant:** All evaluation paths must complete in under **100 microseconds (P99)**. Never introduce network I/O, heavy disk scans, or non-deterministic locking into the fast decision path.
2. **Fail-Closed Guarantee:** When parsing errors or ambiguous directives arise, the default response must be `VetoDenied` or `EscalateParliament`, never silent approval.
3. **Cryptographic Proof Trail:** Every decision MUST generate a valid SHA-256 audit block chained to the previous verdict.

---

## Submitting Pull Requests

1. Fork the repository and create your branch from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```
2. Commit your changes using conventional commit messages:
   - `feat(...)`: New feature or constitutional capability
   - `fix(...)`: Bug fix or security hardening
   - `perf(...)`: Latency optimization
   - `docs(...)`: Documentation updates
3. Ensure CI passes cleanly.
4. Open a Pull Request on GitHub with a clear description of the rationale and benchmark results.
