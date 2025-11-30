"""LNMP utility functions (quant, sanitize, debug, etc.)."""

import lnmp_py_core


def quantize(vector: list[float], scheme: str = "QInt8") -> bytes:
    """Quantize embedding vector to compressed format.
    
    Args:
        vector: Input embedding vector
        scheme: Quantization scheme ("QInt8", "QInt4", "Binary")
    
    Returns:
        Quantized vector as bytes
    
    Example:
        >>> vector = [0.1, 0.2, 0.3]
        >>> quantized = lnmp.utils.quantize(vector, "QInt8")
    """
    return lnmp_py_core.quantize(vector, scheme)


def sanitize(text: str) -> str:
    """Sanitize LNMP text input.
    
    Args:
        text: Raw LNMP text
    
    Returns:
        Sanitized LNMP text
    
    Example:
        >>> raw = "F12= 14532 ; F7=1"
        >>> clean = lnmp.utils.sanitize(raw)
        >>> clean
        'F12=14532;F7=1'
    """
    return lnmp_py_core.sanitize(text)


def debug_explain(text: str) -> str:
    """Get human-readable explanation of LNMP record.
    
    Args:
        text: LNMP formatted text
    
    Returns:
        Explained LNMP with annotations
    
    Example:
        >>> explained = lnmp.utils.debug_explain("F12=14532")
        >>> print(explained)
        F12:i=14532
    """
    return lnmp_py_core.debug_explain(text)
