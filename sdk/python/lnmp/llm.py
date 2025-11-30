"""High-level LLM workflow utilities."""

from typing import Dict, Any, Optional
from . import core, envelope, net


def normalize_and_route(
    text: str,
    source: str,
    *,
    trace_id: Optional[str] = None,
    threshold: float = 0.7,
) -> Dict[str, Any]:
    """Complete workflow: parse, wrap, score, and route.
    
    This is a convenience function that combines all steps needed to
    process raw LNMP text and determine routing.
    
    Args:
        text: LNMP formatted text
        source: Source identifier
        trace_id: Optional trace ID
        threshold: LLM routing threshold
    
    Returns:
        Dictionary containing:
            - record: Parsed Record
            - envelope: Wrapped Envelope
            - score: ContextScore
            - decision: Routing decision string
            - send_to_llm: Boolean indicating LLM routing
    
    Example:
        >>> result = lnmp.llm.normalize_and_route(
        ...     "F12=14532;F7=1",
        ...     source="health-service",
        ...     threshold=0.7
        ... )
        >>> if result["send_to_llm"]:
        ...     send_to_llm(result["envelope"])
    """
    record = core.parse(text)
    env = envelope.wrap(record, source, trace_id=trace_id)
    score = net.context_score(env)
    decision = net.routing_decide(env)
    send_to_llm = score.composite >= threshold
    
    return {
        "record": record,
        "envelope": env,
        "score": score,
        "decision": decision,
        "send_to_llm": send_to_llm,
    }
