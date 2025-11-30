"""LNMP spatial data encoding and streaming."""

import lnmp_py_core
from typing import Tuple


def encode_position3d(x: float, y: float, z: float) -> bytes:
    """Encode a 3D position to LNMP spatial format.
    
    Args:
        x: X coordinate
        y: Y coordinate
        z: Z coordinate
    
    Returns:
        Encoded position as bytes
    
    Example:
        >>> pos_bytes = lnmp.spatial.encode_position3d(1.5, 2.5, 3.5)
        >>> len(pos_bytes)
        13
    """
    return lnmp_py_core.spatial_encode_position3d(x, y, z)


def decode_position3d(data: bytes) -> Tuple[float, float, float]:
    """Decode a 3D position from LNMP spatial format.
    
    Args:
        data: Encoded spatial data
    
    Returns:
        Tuple of (x, y, z) coordinates
    
    Example:
        >>> data = lnmp.spatial.encode_position3d(1.5, 2.5, 3.5)
        >>> x, y, z = lnmp.spatial.decode_position3d(data)
        >>> (x, y, z)
        (1.5, 2.5, 3.5)
    """
    return lnmp_py_core.spatial_decode_position3d(data)
