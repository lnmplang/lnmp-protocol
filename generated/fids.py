"""
LNMP Field ID Constants

Auto-generated from registry/fids.yaml v1.0.0
Generated: 2025-12-17T01:16:15.583124

DO NOT EDIT MANUALLY
"""

from typing import Dict


class FID:
    """LNMP Official Field IDs."""

    # F1: Unique entity identifier within a context
    ENTITY_ID: int = 1

    # F2: Unix timestamp in milliseconds (UTC)
    TIMESTAMP: int = 2

    # F3: Schema or data version number
    VERSION: int = 3

    # F4: Monotonic sequence number for ordering
    SEQUENCE: int = 4

    # F5: Origin identifier (service, device, node)
    SOURCE: int = 5

    # F7: Active/inactive boolean flag
    IS_ACTIVE: int = 7

    # F8: Validity boolean flag
    IS_VALID: int = 8

    # F12: User identifier
    USER_ID: int = 12

    # F13: Session identifier
    SESSION_ID: int = 13

    # F20: Human-readable name
    NAME: int = 20

    # F21: Short label or tag
    LABEL: int = 21

    # F22: Long-form description text
    DESCRIPTION: int = 22

    # F23: List of role identifiers
    ROLES: int = 23

    # F24: List of tags for categorization
    TAGS: int = 24

    # F30: Generic count value
    COUNT: int = 30

    # F31: Zero-based index
    INDEX: int = 31

    # F32: Priority level (0 = lowest)
    PRIORITY: int = 32

    # F40: Generic floating-point value
    VALUE: int = 40

    # F41: Score or rating (typically 0.0-1.0)
    SCORE: int = 41

    # F42: Confidence level (0.0-1.0)
    CONFIDENCE: int = 42

    # F50: Status string (pending, active, completed, etc.)
    STATUS: int = 50

    # F51: Error code (0 = no error)
    ERROR_CODE: int = 51

    # F52: Human-readable error message
    ERROR_MESSAGE: int = 52

    # F60: Generic integer array for numeric data
    INT_VALUES: int = 60

    # F61: Generic float array for measurement data
    FLOAT_VALUES: int = 61

    # F62: Generic boolean array for flag sets
    BOOL_FLAGS: int = 62

    # F70: Generic nested record container
    NESTED_DATA: int = 70

    # F71: Array of nested records
    RECORD_LIST: int = 71

    # F256: [x, y, z] position coordinates in meters
    POSITION: int = 256

    # F257: [roll, pitch, yaw] Euler angles in radians
    ROTATION: int = 257

    # F258: [vx, vy, vz] linear velocity
    VELOCITY: int = 258

    # F259: [ax, ay, az] linear acceleration
    ACCELERATION: int = 259

    # F260: [w, x, y, z] rotation quaternion
    QUATERNION: int = 260

    # F261: [min_x, min_y, min_z, max_x, max_y, max_z]
    BOUNDING_BOX: int = 261

    # F512: Vector embedding (variable dimension)
    EMBEDDING: int = 512

    # F513: Embedding model identifier
    EMBEDDING_MODEL: int = 513

    # F514: Embedding dimension size
    EMBEDDING_DIM: int = 514

    # F768: Temperature in Celsius
    TEMPERATURE: int = 768

    # F769: Relative humidity percentage
    HUMIDITY: int = 769

    # F770: Pressure in Pascals
    PRESSURE: int = 770

    # F771: Battery charge percentage
    BATTERY_LEVEL: int = 771

    # F1024: Message classification (Event, State, Command, Query, Alert)
    MESSAGE_KIND: int = 1024

    # F1025: Time-to-live in milliseconds
    TTL: int = 1025

    # F1026: QoS priority level (0-255)
    QOS_PRIORITY: int = 1026


# Reverse lookup: FID number to name
FID_NAMES: Dict[int, str] = {
    1: 'entity_id',
    2: 'timestamp',
    3: 'version',
    4: 'sequence',
    5: 'source',
    7: 'is_active',
    8: 'is_valid',
    12: 'user_id',
    13: 'session_id',
    20: 'name',
    21: 'label',
    22: 'description',
    23: 'roles',
    24: 'tags',
    30: 'count',
    31: 'index',
    32: 'priority',
    40: 'value',
    41: 'score',
    42: 'confidence',
    50: 'status',
    51: 'error_code',
    52: 'error_message',
    60: 'int_values',
    61: 'float_values',
    62: 'bool_flags',
    70: 'nested_data',
    71: 'record_list',
    256: 'position',
    257: 'rotation',
    258: 'velocity',
    259: 'acceleration',
    260: 'quaternion',
    261: 'bounding_box',
    512: 'embedding',
    513: 'embedding_model',
    514: 'embedding_dim',
    768: 'temperature',
    769: 'humidity',
    770: 'pressure',
    771: 'battery_level',
    1024: 'message_kind',
    1025: 'ttl',
    1026: 'qos_priority',
}
