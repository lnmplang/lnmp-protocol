"""Core LNMP parsing and encoding functionality."""

from typing import Union
import lnmp_py_core

class Record:
    """High-level wrapper for LNMP records."""
    
    def __init__(self, _inner=None):
        if _inner is None:
            raise ValueError("Use parse() to create Record instances")
        self._inner = _inner
    
    def encode(self) -> str:
        """Encode record to LNMP text format."""
        return lnmp_py_core.encode(self._inner)
    
    def encode_binary(self) -> bytes:
        """Encode record to binary LNMP format."""
        return lnmp_py_core.encode_binary(self._inner)
    
    def __repr__(self) -> str:
        return f"Record({self.encode()!r})"


def parse(text: str) -> Record:
    """Parse LNMP text format into a Record.
    
    Args:
        text: LNMP formatted text (e.g., "F12=14532;F7=1")
    
    Returns:
        Record instance
    
    Example:
        >>> record = lnmp.core.parse("F12=14532;F7=1")
        >>> record.encode()
        'F12=14532;F7=1'
    """
    return Record(_inner=lnmp_py_core.parse(text))


def decode_binary(data: bytes) -> Record:
    """Decode binary LNMP format into a Record.
    
    Args:
        data: Binary LNMP data
    
    Returns:
        Record instance
    """
    return Record(_inner=lnmp_py_core.decode_binary(data))
