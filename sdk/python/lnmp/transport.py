"""LNMP Transport bindings.

This module provides helpers to map LNMP Envelopes to and from transport-specific
metadata formats (e.g., HTTP headers).
"""

import lnmp_py_core
from typing import Dict, Optional
from .envelope import Envelope

def to_http_headers(envelope: Envelope) -> Dict[str, str]:
    """Convert an LNMP Envelope to HTTP headers.
    
    Maps envelope metadata to standard X-LNMP-* headers and W3C Trace Context.
    
    Args:
        envelope: The LNMP envelope to convert.
        
    Returns:
        Dictionary of HTTP headers.
        
    Example:
        >>> headers = lnmp.transport.to_http_headers(envelope)
        >>> headers['x-lnmp-source']
        'my-service'
    """
    return lnmp_py_core.transport_to_http_headers(envelope._inner)

def from_http_headers(headers: Dict[str, str]) -> Envelope:
    """Create an LNMP Envelope from HTTP headers.
    
    Extracts metadata from X-LNMP-* headers and W3C Trace Context.
    Note: The returned envelope contains an empty record. You should typically
    parse the body separately and combine it with the metadata.
    
    Args:
        headers: Dictionary of HTTP headers.
        
    Returns:
        LNMP Envelope with extracted metadata.
        
    Example:
        >>> envelope = lnmp.transport.from_http_headers(request.headers)
        >>> envelope.source
        'my-service'
    """
    # Create internal envelope from headers
    inner = lnmp_py_core.transport_from_http_headers(headers)
    return Envelope(_inner=inner)
