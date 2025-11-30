"""LNMP embedding vector operations."""

import lnmp_py_core
from typing import List, Tuple


def delta(base: List[float], updated: List[float]) -> Tuple[int, bytes]:
    """Compute delta between two embedding vectors.
    
    Args:
        base: The base vector (list of floats).
        updated: The updated vector (list of floats).
        
    Returns:
        Tuple containing:
        - change_count (int): Number of changed dimensions
        - encoded_delta (bytes): Binary encoded delta
        
    Example:
        >>> base = [0.1, 0.2, 0.3]
        >>> updated = [0.1, 0.25, 0.3]
        >>> count, data = lnmp.embedding.delta(base, updated)
    """
    return lnmp_py_core.embedding_delta(base, updated)


def apply_delta(base: List[float], delta: bytes) -> List[float]:
    """Apply a delta update to a base vector.
    
    Args:
        base: The original vector (list of floats).
        delta: The binary delta data.
        
    Returns:
        The updated vector (list of floats).
        
    Example:
        >>> base = [0.1, 0.2, 0.3]
        >>> delta = ... # bytes from delta()
        >>> updated = lnmp.embedding.apply_delta(base, delta)
    """
    return lnmp_py_core.embedding_apply_delta(base, delta)
