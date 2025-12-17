/**
 * LNMP Field ID Constants
 *
 * Auto-generated from registry/fids.yaml v1.0.0
 * Generated: 2025-12-17T01:16:15.582915
 *
 * DO NOT EDIT MANUALLY
 */

/** Field ID type */
export type Fid = number;

/** LNMP Official Field IDs */
export const FID = {
  /** F1: Unique entity identifier within a context */
  ENTITY_ID: 1,
  /** F2: Unix timestamp in milliseconds (UTC) */
  TIMESTAMP: 2,
  /** F3: Schema or data version number */
  VERSION: 3,
  /** F4: Monotonic sequence number for ordering */
  SEQUENCE: 4,
  /** F5: Origin identifier (service, device, node) */
  SOURCE: 5,
  /** F7: Active/inactive boolean flag */
  IS_ACTIVE: 7,
  /** F8: Validity boolean flag */
  IS_VALID: 8,
  /** F12: User identifier */
  USER_ID: 12,
  /** F13: Session identifier */
  SESSION_ID: 13,
  /** F20: Human-readable name */
  NAME: 20,
  /** F21: Short label or tag */
  LABEL: 21,
  /** F22: Long-form description text */
  DESCRIPTION: 22,
  /** F23: List of role identifiers */
  ROLES: 23,
  /** F24: List of tags for categorization */
  TAGS: 24,
  /** F30: Generic count value */
  COUNT: 30,
  /** F31: Zero-based index */
  INDEX: 31,
  /** F32: Priority level (0 = lowest) */
  PRIORITY: 32,
  /** F40: Generic floating-point value */
  VALUE: 40,
  /** F41: Score or rating (typically 0.0-1.0) */
  SCORE: 41,
  /** F42: Confidence level (0.0-1.0) */
  CONFIDENCE: 42,
  /** F50: Status string (pending, active, completed, etc.) */
  STATUS: 50,
  /** F51: Error code (0 = no error) */
  ERROR_CODE: 51,
  /** F52: Human-readable error message */
  ERROR_MESSAGE: 52,
  /** F60: Generic integer array for numeric data */
  INT_VALUES: 60,
  /** F61: Generic float array for measurement data */
  FLOAT_VALUES: 61,
  /** F62: Generic boolean array for flag sets */
  BOOL_FLAGS: 62,
  /** F70: Generic nested record container */
  NESTED_DATA: 70,
  /** F71: Array of nested records */
  RECORD_LIST: 71,
  /** F256: [x, y, z] position coordinates in meters */
  POSITION: 256,
  /** F257: [roll, pitch, yaw] Euler angles in radians */
  ROTATION: 257,
  /** F258: [vx, vy, vz] linear velocity */
  VELOCITY: 258,
  /** F259: [ax, ay, az] linear acceleration */
  ACCELERATION: 259,
  /** F260: [w, x, y, z] rotation quaternion */
  QUATERNION: 260,
  /** F261: [min_x, min_y, min_z, max_x, max_y, max_z] */
  BOUNDING_BOX: 261,
  /** F512: Vector embedding (variable dimension) */
  EMBEDDING: 512,
  /** F513: Embedding model identifier */
  EMBEDDING_MODEL: 513,
  /** F514: Embedding dimension size */
  EMBEDDING_DIM: 514,
  /** F768: Temperature in Celsius */
  TEMPERATURE: 768,
  /** F769: Relative humidity percentage */
  HUMIDITY: 769,
  /** F770: Pressure in Pascals */
  PRESSURE: 770,
  /** F771: Battery charge percentage */
  BATTERY_LEVEL: 771,
  /** F1024: Message classification (Event, State, Command, Query, Alert) */
  MESSAGE_KIND: 1024,
  /** F1025: Time-to-live in milliseconds */
  TTL: 1025,
  /** F1026: QoS priority level (0-255) */
  QOS_PRIORITY: 1026,
} as const;

/** Type for FID keys */
export type FidKey = keyof typeof FID;

/** Reverse lookup: FID number to name */
export const FID_NAMES: Record<number, string> = {
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
};
