import ctypes
import json
import os
import sys
from pathlib import Path
from typing import Optional, Dict, Any

from .types import DeciderVerdict, VerdictStatus, ActionDirective, ConfidenceLevel

def _find_library() -> str:
    # 1. Check local directory / target/release
    candidates = [
        Path(__file__).parent / "libseed_core.so",
        Path(__file__).parent.parent.parent / "target" / "release" / "libseed_core.so",
        Path(__file__).parent.parent.parent / "target" / "debug" / "libseed_core.so",
        Path("/usr/local/lib/libseed_core.so"),
        Path("/usr/lib/libseed_core.so"),
    ]
    for c in candidates:
        if c.exists():
            return str(c)
    raise FileNotFoundError(
        "Could not find libseed_core.so. Please run 'cargo build --release' first."
    )

class SeedDecider:
    """
    Sub-microsecond deterministic 2-stage execution gatekeeper and AGI decider
    powered by native Rust.
    """
    def __init__(self, profile: str = "HOST_SOVEREIGN", target: str = "host"):
        self._lib_path = _find_library()
        self._lib = ctypes.CDLL(self._lib_path)

        # Setup FFI function signatures
        self._lib.seed_decider_create.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self._lib.seed_decider_create.restype = ctypes.c_void_p

        self._lib.seed_decider_evaluate.argtypes = [
            ctypes.c_void_p,
            ctypes.c_char_p,
            ctypes.c_char_p,
        ]
        self._lib.seed_decider_evaluate.restype = ctypes.c_char_p

        self._lib.seed_decider_free.argtypes = [ctypes.c_void_p]
        self._lib.seed_decider_free.restype = None

        self._lib.seed_string_free.argtypes = [ctypes.c_char_p]
        self._lib.seed_string_free.restype = None

        # Create decider instance
        self._ptr = self._lib.seed_decider_create(
            profile.encode("utf-8"),
            target.encode("utf-8"),
        )
        if not self._ptr:
            raise RuntimeError("Failed to allocate native SeedDecider in Rust.")

    def evaluate(
        self,
        intent: str,
        action_type: str,
        target: Optional[str] = None,
        lid: float = 0.25,
        etm: float = 0.90,
        step_id: int = 1,
    ) -> DeciderVerdict:
        """
        Evaluate candidate action step against constitutional JFP policies and LID geometry.
        Latency: ~15 to 90 microseconds.
        """
        # Map action type string
        type_clean = action_type.lower()
        if type_clean in ("command_exec", "command", "exec"):
            act_type = "CommandExec"
            args = {"command": target} if target else {}
        elif type_clean in ("file_read", "read"):
            act_type = "FileRead"
            args = {"path": target} if target else {}
        elif type_clean in ("file_write", "write"):
            act_type = "FileWrite"
            args = {"path": target} if target else {}
        elif type_clean in ("network_fetch", "network", "curl", "fetch"):
            act_type = "NetworkFetch"
            args = {"url": target, "path": target} if target else {}
        elif type_clean in ("browser_action", "browser", "browse"):
            act_type = "BrowserAction"
            args = {"url": target} if target else {}
        else:
            act_type = "PlanStep"
            args = {"target": target} if target else {}

        candidate_obj = {
            "step_id": step_id,
            "intent": intent,
            "action_type": act_type,
            "action_args": args,
            "planner_thought": "Python SDK Evaluation",
        }

        geom_obj = {
            "lid": lid,
            "cos_gap": 0.45,
            "etm": etm,
            "activated_domains": ["code", "jfp"],
            "activated_ages": [1, 2],
        }

        cand_bytes = json.dumps(candidate_obj).encode("utf-8")
        geom_bytes = json.dumps(geom_obj).encode("utf-8")

        res_ptr = self._lib.seed_decider_evaluate(self._ptr, cand_bytes, geom_bytes)
        if not res_ptr:
            raise RuntimeError("Null response returned from native SeedDecider.")

        raw_str = res_ptr.decode("utf-8")
        data = json.loads(raw_str)

        return DeciderVerdict(
            step_id=data["step_id"],
            verdict=VerdictStatus(data["verdict"]),
            action=ActionDirective(data["action"]),
            confidence=ConfidenceLevel(data["confidence"]),
            reason=data["reason"],
            latency_us=data["latency_us"],
            audit_hash=data["audit_hash"],
            timestamp=data["timestamp"],
        )

    def __del__(self):
        if getattr(self, "_ptr", None) and getattr(self, "_lib", None):
            self._lib.seed_decider_free(self._ptr)
            self._ptr = None
