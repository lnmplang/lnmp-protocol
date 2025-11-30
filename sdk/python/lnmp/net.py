"""LNMP network routing and context scoring."""

from typing import Dict
import lnmp_py_core
from .envelope import Envelope


class ContextScore:
    """Context scoring result."""
    
    def __init__(self, data: Dict[str, float]):
        self.composite = data.get("composite", 0.0)
        self.freshness = data.get("freshness", 0.0)
        self.importance = data.get("importance", 0.0)
        self.risk = data.get("risk", 0.0)
        self.confidence = data.get("confidence", 0.0)
    
    def __repr__(self) -> str:
        return (
            f"ContextScore(composite={self.composite:.3f}, "
            f"freshness={self.freshness:.3f}, "
            f"importance={self.importance:.3f}, "
            f"risk={self.risk:.3f}, "
            f"confidence={self.confidence:.3f})"
        )


def context_score(envelope: Envelope) -> ContextScore:
    """Calculate context score for an envelope.
    
    Args:
        envelope: Envelope to score
    
    Returns:
        ContextScore with composite and component scores
    """
    data = lnmp_py_core.context_score(envelope._inner)
    return ContextScore(data)


def routing_decide(envelope: Envelope) -> str:
    """Decide routing for an envelope.
    
    Args:
        envelope: Envelope to route
    
    Returns:
        Routing decision ("SendToLLM", "Drop", "ProcessLocally", etc.)
    """
    return lnmp_py_core.routing_decide(envelope._inner)


def should_send_to_llm(envelope: Envelope, *, threshold: float = 0.7) -> bool:
    """Determine if envelope should be sent to LLM based on scoring.
    
    Args:
        envelope: Envelope to evaluate
        threshold: Minimum composite score for LLM routing (0.0-1.0)
    
    Returns:
        True if envelope should be sent to LLM
    
    Example:
        >>> if lnmp.net.should_send_to_llm(env, threshold=0.7):
        ...     send_to_llm(env)
        ... else:
        ...     process_locally(env)
    """
    score = context_score(envelope)
    return score.composite >= threshold
