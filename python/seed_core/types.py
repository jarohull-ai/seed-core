from dataclasses import dataclass
from enum import Enum
from typing import Optional, Dict, Any, List

class VerdictStatus(str, Enum):
    APPROVED = "Approved"
    VETO_DENIED = "VetoDenied"
    ESCALATE_PARLIAMENT = "EscalateParliament"

class ActionDirective(str, Enum):
    EXECUTE = "Execute"
    REFUSE = "Refuse"
    ESCALATE = "Escalate"

class ConfidenceLevel(str, Enum):
    LOW = "Low"
    MID = "Mid"
    HIGH = "High"

@dataclass
class DeciderVerdict:
    step_id: int
    verdict: VerdictStatus
    action: ActionDirective
    confidence: ConfidenceLevel
    reason: str
    latency_us: int
    audit_hash: str
    timestamp: str

    @property
    def is_approved(self) -> bool:
        return self.verdict == VerdictStatus.APPROVED

    @property
    def is_vetoed(self) -> bool:
        return self.verdict == VerdictStatus.VETO_DENIED

    @property
    def requires_escalation(self) -> bool:
        return self.verdict == VerdictStatus.ESCALATE_PARLIAMENT
