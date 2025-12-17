//! LNMP Field ID Constants
//!
//! Auto-generated from registry/fids.yaml v1.0.0
//! Generated: 2025-12-17T01:16:15.582528
//!
//! DO NOT EDIT MANUALLY

#![allow(dead_code)]

/// Field ID type alias
pub type Fid = u16;

// =============================================================================
// CORE FIELDS (0-255) - LOCKED
// =============================================================================

/// F1: Unique entity identifier within a context
pub const FID_ENTITY_ID: Fid = 1;

/// F2: Unix timestamp in milliseconds (UTC) (unit: ms)
pub const FID_TIMESTAMP: Fid = 2;

/// F3: Schema or data version number
pub const FID_VERSION: Fid = 3;

/// F4: Monotonic sequence number for ordering
pub const FID_SEQUENCE: Fid = 4;

/// F5: Origin identifier (service, device, node)
pub const FID_SOURCE: Fid = 5;

/// F7: Active/inactive boolean flag
pub const FID_IS_ACTIVE: Fid = 7;

/// F8: Validity boolean flag
pub const FID_IS_VALID: Fid = 8;

/// F12: User identifier
pub const FID_USER_ID: Fid = 12;

/// F13: Session identifier
pub const FID_SESSION_ID: Fid = 13;

/// F20: Human-readable name
pub const FID_NAME: Fid = 20;

/// F21: Short label or tag
pub const FID_LABEL: Fid = 21;

/// F22: Long-form description text
pub const FID_DESCRIPTION: Fid = 22;

/// F23: List of role identifiers
pub const FID_ROLES: Fid = 23;

/// F24: List of tags for categorization
pub const FID_TAGS: Fid = 24;

/// F30: Generic count value
pub const FID_COUNT: Fid = 30;

/// F31: Zero-based index
pub const FID_INDEX: Fid = 31;

/// F32: Priority level (0 = lowest)
pub const FID_PRIORITY: Fid = 32;

/// F40: Generic floating-point value
pub const FID_VALUE: Fid = 40;

/// F41: Score or rating (typically 0.0-1.0)
pub const FID_SCORE: Fid = 41;

/// F42: Confidence level (0.0-1.0)
pub const FID_CONFIDENCE: Fid = 42;

/// F50: Status string (pending, active, completed, etc.)
pub const FID_STATUS: Fid = 50;

/// F51: Error code (0 = no error)
pub const FID_ERROR_CODE: Fid = 51;

/// F52: Human-readable error message
pub const FID_ERROR_MESSAGE: Fid = 52;

/// F60: Generic integer array for numeric data
pub const FID_INT_VALUES: Fid = 60;

/// F61: Generic float array for measurement data
pub const FID_FLOAT_VALUES: Fid = 61;

/// F62: Generic boolean array for flag sets
pub const FID_BOOL_FLAGS: Fid = 62;

/// F70: Generic nested record container
pub const FID_NESTED_DATA: Fid = 70;

/// F71: Array of nested records
pub const FID_RECORD_LIST: Fid = 71;


// =============================================================================
// STANDARD FIELDS
// =============================================================================

/// F256: [x, y, z] position coordinates in meters (unit: m)
pub const FID_POSITION: Fid = 256;

/// F257: [roll, pitch, yaw] Euler angles in radians (unit: rad)
pub const FID_ROTATION: Fid = 257;

/// F258: [vx, vy, vz] linear velocity (unit: m/s)
pub const FID_VELOCITY: Fid = 258;

/// F259: [ax, ay, az] linear acceleration (unit: m/s²)
pub const FID_ACCELERATION: Fid = 259;

/// F260: [w, x, y, z] rotation quaternion
pub const FID_QUATERNION: Fid = 260;

/// F261: [min_x, min_y, min_z, max_x, max_y, max_z] (unit: m)
pub const FID_BOUNDING_BOX: Fid = 261;

/// F512: Vector embedding (variable dimension)
pub const FID_EMBEDDING: Fid = 512;

/// F513: Embedding model identifier
pub const FID_EMBEDDING_MODEL: Fid = 513;

/// F514: Embedding dimension size
pub const FID_EMBEDDING_DIM: Fid = 514;

/// F768: Temperature in Celsius (unit: °C)
pub const FID_TEMPERATURE: Fid = 768;

/// F769: Relative humidity percentage (unit: %)
pub const FID_HUMIDITY: Fid = 769;

/// F770: Pressure in Pascals (unit: Pa)
pub const FID_PRESSURE: Fid = 770;

/// F771: Battery charge percentage (unit: %)
pub const FID_BATTERY_LEVEL: Fid = 771;

/// F1024: Message classification (Event, State, Command, Query, Alert)
pub const FID_MESSAGE_KIND: Fid = 1024;

/// F1025: Time-to-live in milliseconds (unit: ms)
pub const FID_TTL: Fid = 1025;

/// F1026: QoS priority level (0-255)
pub const FID_QOS_PRIORITY: Fid = 1026;
