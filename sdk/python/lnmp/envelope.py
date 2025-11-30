"""LNMP envelope functionality for message metadata."""

from typing import Optional
from datetime import datetime
import lnmp_py_core
from .core import Record


class Envelope:
    """High-level wrapper for LNMP envelopes."""
    
    def __init__(self, _inner=None):
        if _inner is None:
            raise ValueError("Use wrap() to create Envelope instances")
        self._inner = _inner

    @property
    def source(self) -> Optional[str]:
        """Get the source identifier."""
        return self._inner.source

    @property
    def trace_id(self) -> Optional[str]:
        """Get the trace ID."""
        return self._inner.trace_id

    @property
    def timestamp(self) -> Optional[int]:
        """Get the timestamp."""
        return self._inner.timestamp


def wrap(
    record: Record,
    source: str,
    *,
    timestamp_ms: Optional[int] = None,
    trace_id: Optional[str] = None,
) -> Envelope:
    """Wrap a record with envelope metadata.
    
    Args:
        record: LNMP record to wrap
        source: Source identifier (e.g., "health-service")
        timestamp_ms: Optional timestamp in milliseconds (defaults to now)
        trace_id: Optional trace ID for request tracking
    
    Returns:
        Envelope instance
    
    Example:
        >>> record = lnmp.core.parse("F12=14532")
        >>> env = lnmp.envelope.wrap(
        ...     record,
        ...     source="my-service",
        ...     trace_id="trace-123"
        ... )
    """
    if timestamp_ms is None:
        timestamp_ms = int(datetime.now().timestamp() * 1000)
    
    return Envelope(
        _inner=lnmp_py_core.envelope_wrap(
            record._inner,
            source,
            timestamp_ms,
            trace_id,
        )
    )
