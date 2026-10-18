# Security Policy

The **SEED-Core** team takes the security and integrity of autonomous agent decision-making extremely seriously. As the Stage-2 Gatekeeper standing between untrusted LLMs and sovereign execution environments, deterministic safety is our primary invariant.

---

## Supported Versions

Only the latest release receive security patches and updates:

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| `0.1.x` | :white_check_mark: | Active Mainline |
| `< 0.1` | :x:                | Deprecated |

---

## Reporting a Vulnerability (Responsible Disclosure)

If you discover a security vulnerability, gatekeeper bypass, or memory integrity flaw within **SEED-Core**:

1. **Do NOT open a public GitHub issue.**
2. Send an email to **`jarohull@gmail.com`** with:
   - A clear description of the vulnerability or policy bypass vector.
   - Minimal reproduction code (Rust, Python, or CLI command).
   - Expected behavior vs. actual behavior observed.
   - Impact assessment (e.g. host escape, unauthorized file mutation, denial of service).
3. **Response Time:** We acknowledge security disclosures within **24 hours** and provide a mitigation plan / patch timeline within **72 hours**.

---

## Security Architecture & Invariants

SEED-Core is built on native Rust to enforce strict constitutional guarantees:
- **Zero Raw LLM Execution Privileges:** Stage 1 LLMs only emit candidate directives. The native Rust engine evaluates all mutations deterministically before execution.
- **Memory Safety:** Business logic and constitutional policies are 100% memory-safe Rust with zero heap corruption vulnerabilities. C-FFI boundaries are strictly validated for null pointers and string lengths.
- **Cryptographic Audit Trail:** All evaluations, approvals, and vetoes append to a continuous SHA-256 block chain, ensuring tamper-evident accountability for SOC2, ISO 42001, and EU AI Act compliance.
- **Regex Denial-of-Service Defense:** All constitutional patterns are pre-compiled and bounded to linear time guarantees via the Rust `regex` engine.

---

## Bug Bounty & Responsible Disclosure Recognition

Security researchers who responsibly disclose verified gatekeeper bypasses or memory safety flaws will be publicly credited in our release notes and Hall of Fame (unless anonymity is requested).
