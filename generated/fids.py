"""
LNMP Field ID Constants

Auto-generated from registry/fids.yaml v1.2.0
Generated: 2025-12-18T22:38:44.883502

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

    # F80: Distributed trace identifier (W3C Trace Context compatible)
    TRACE_ID: int = 80

    # F81: Span identifier within a trace
    SPAN_ID: int = 81

    # F82: Parent span identifier for trace hierarchy
    PARENT_SPAN_ID: int = 82

    # F83: Trace flags (sampled, random, etc.)
    TRACE_FLAGS: int = 83

    # F84: Originating service name (OpenTelemetry convention)
    SERVICE_NAME: int = 84

    # F85: Originating service version
    SERVICE_VERSION: int = 85

    # F100: Unix timestamp in nanoseconds (high-precision)
    TIMESTAMP_NS: int = 100

    # F101: Duration/elapsed time in milliseconds
    DURATION_MS: int = 101

    # F102: Duration/elapsed time in nanoseconds
    DURATION_NS: int = 102

    # F103: Start timestamp in milliseconds
    START_TIME: int = 103

    # F104: End timestamp in milliseconds
    END_TIME: int = 104

    # F105: Creation timestamp in milliseconds
    CREATED_AT: int = 105

    # F106: Last update timestamp in milliseconds
    UPDATED_AT: int = 106

    # F120: Event type classification (CloudEvents type)
    EVENT_TYPE: int = 120

    # F121: Event origin URI (CloudEvents source)
    EVENT_SOURCE: int = 121

    # F122: Request correlation identifier for distributed systems
    CORRELATION_ID: int = 122

    # F123: Unique request identifier
    REQUEST_ID: int = 123

    # F124: Transaction identifier for business processes
    TRANSACTION_ID: int = 124

    # F125: Multi-tenancy identifier
    TENANT_ID: int = 125

    # F126: Organization/workspace identifier
    ORGANIZATION_ID: int = 126

    # F127: Physical device identifier
    DEVICE_ID: int = 127

    # F128: Data stream identifier
    STREAM_ID: int = 128

    # F129: Communication channel identifier
    CHANNEL_ID: int = 129

    # F130: Authentication token (JWT, OAuth, etc.)
    AUTH_TOKEN: int = 130

    # F131: Token refresh credential
    REFRESH_TOKEN: int = 131

    # F132: Token expiration timestamp in milliseconds
    TOKEN_EXPIRY: int = 132

    # F133: OAuth2 scopes or permission scopes
    SCOPES: int = 133

    # F134: Access permission identifiers
    PERMISSIONS: int = 134

    # F135: API key for service authentication
    API_KEY: int = 135

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

    # F280: WGS84 latitude (-90 to 90 degrees)
    LATITUDE: int = 280

    # F281: WGS84 longitude (-180 to 180 degrees)
    LONGITUDE: int = 281

    # F282: Altitude above sea level in meters
    ALTITUDE: int = 282

    # F283: Compass heading (0-360 degrees, 0=North)
    HEADING: int = 283

    # F284: Ground speed in meters per second
    GROUND_SPEED: int = 284

    # F285: Position accuracy radius in meters
    POSITION_ACCURACY: int = 285

    # F286: GeoHash encoded location string
    GEO_HASH: int = 286

    # F300: Full URL/URI
    URL: int = 300

    # F301: Host name or IP address
    HOSTNAME: int = 301

    # F302: Network port number (0-65535)
    PORT: int = 302

    # F303: IPv4 or IPv6 address
    IP_ADDRESS: int = 303

    # F304: HTTP method (GET, POST, PUT, DELETE, etc.)
    HTTP_METHOD: int = 304

    # F305: HTTP response status code (200, 404, 500, etc.)
    HTTP_STATUS_CODE: int = 305

    # F306: HTTP User-Agent header value
    USER_AGENT: int = 306

    # F307: MIME content type (application/json, etc.)
    CONTENT_TYPE: int = 307

    # F308: Content size in bytes
    CONTENT_LENGTH: int = 308

    # F309: Content encoding (utf-8, gzip, etc.)
    ENCODING: int = 309

    # F512: Vector embedding (variable dimension)
    EMBEDDING: int = 512

    # F513: Embedding model identifier
    EMBEDDING_MODEL: int = 513

    # F514: Embedding dimension size
    EMBEDDING_DIM: int = 514

    # F520: [wx, wy, wz] angular velocity
    ANGULAR_VELOCITY: int = 520

    # F521: [ax, ay, az] IMU linear acceleration
    LINEAR_ACCELERATION: int = 521

    # F522: [mx, my, mz] magnetic field in Tesla
    MAGNETIC_FIELD: int = 522

    # F523: [w, x, y, z] orientation quaternion from IMU
    ORIENTATION: int = 523

    # F524: Robot joint positions in radians
    JOINT_POSITIONS: int = 524

    # F525: Robot joint velocities
    JOINT_VELOCITIES: int = 525

    # F526: Robot joint torques/forces
    JOINT_EFFORTS: int = 526

    # F527: Robot joint names
    JOINT_NAMES: int = 527

    # F528: Navigation waypoints as records
    WAYPOINTS: int = 528

    # F529: Flattened 3D point cloud [x,y,z,x,y,z,...]
    POINT_CLOUD: int = 529

    # F530: [vx,vy,vz,wx,wy,wz] linear+angular velocity (ROS Twist)
    TWIST: int = 530

    # F768: Temperature in Celsius
    TEMPERATURE: int = 768

    # F769: Relative humidity percentage
    HUMIDITY: int = 769

    # F770: Pressure in Pascals
    PRESSURE: int = 770

    # F771: Battery charge percentage
    BATTERY_LEVEL: int = 771

    # F772: Light level in lux
    LUMINOSITY: int = 772

    # F773: Sound level in decibels
    NOISE_LEVEL: int = 773

    # F774: CO2 concentration in parts per million
    CO2_LEVEL: int = 774

    # F775: PM2.5 particulate matter concentration
    PM25: int = 775

    # F776: PM10 particulate matter concentration
    PM10: int = 776

    # F777: Volatile organic compounds level
    VOC: int = 777

    # F778: UV radiation index (0-11+)
    UV_INDEX: int = 778

    # F779: Wind speed in meters per second
    WIND_SPEED: int = 779

    # F780: Wind direction (0-360 degrees, 0=North)
    WIND_DIRECTION: int = 780

    # F781: Rainfall accumulation in millimeters
    RAINFALL: int = 781

    # F782: Soil moisture percentage
    SOIL_MOISTURE: int = 782

    # F783: pH level (0-14 scale)
    PH_LEVEL: int = 783

    # F784: Signal strength (RSSI) in dBm
    SIGNAL_STRENGTH: int = 784

    # F1024: Message classification (Event, State, Command, Query, Alert)
    MESSAGE_KIND: int = 1024

    # F1025: Time-to-live in milliseconds
    TTL: int = 1025

    # F1026: QoS priority level (0-255)
    QOS_PRIORITY: int = 1026

    # F1027: Number of delivery retries
    RETRY_COUNT: int = 1027

    # F1028: Message delivery status (pending, delivered, failed)
    DELIVERY_STATUS: int = 1028

    # F1029: Whether acknowledgment is required
    ACK_REQUIRED: int = 1029

    # F1030: Schema identifier for payload (CloudEvents dataschema)
    PAYLOAD_SCHEMA: int = 1030

    # F1031: Compression algorithm (none, gzip, lz4, zstd)
    COMPRESSION: int = 1031

    # F1032: Encryption algorithm (none, aes256, chacha20)
    ENCRYPTION: int = 1032

    # F1040: Message queue topic name (Kafka, RabbitMQ, etc.)
    TOPIC: int = 1040

    # F1041: Kafka partition identifier
    PARTITION_ID: int = 1041

    # F1042: Message offset within partition
    OFFSET: int = 1042

    # F1043: Consumer group identifier
    CONSUMER_GROUP: int = 1043

    # F1044: Message partitioning key
    MESSAGE_KEY: int = 1044

    # F1045: Message broker identifier
    BROKER_ID: int = 1045

    # F1100: ML model identifier
    MODEL_ID: int = 1100

    # F1101: ML model version
    MODEL_VERSION: int = 1101

    # F1102: Model inference time in milliseconds
    INFERENCE_TIME: int = 1102

    # F1103: Model prediction probabilities
    PREDICTIONS: int = 1103

    # F1104: Predicted class index
    PREDICTED_CLASS: int = 1104

    # F1105: Class label names
    CLASS_LABELS: int = 1105

    # F1106: Input feature vector
    FEATURES: int = 1106

    # F1107: Attention/importance weights
    ATTENTION_WEIGHTS: int = 1107

    # F1108: Tokenized input IDs
    TOKEN_IDS: int = 1108

    # F1109: Input tokens for NLP
    INPUT_TOKENS: int = 1109

    # F1110: Output tokens for NLP
    OUTPUT_TOKENS: int = 1110

    # F1111: LLM prompt text
    PROMPT: int = 1111

    # F1112: LLM completion text
    COMPLETION: int = 1112

    # F1113: LLM sampling temperature (0.0-2.0)
    LLM_TEMPERATURE: int = 1113

    # F1114: Maximum output tokens for LLM
    MAX_TOKENS: int = 1114

    # F1200: Video/image width in pixels
    RESOLUTION_WIDTH: int = 1200

    # F1201: Video/image height in pixels
    RESOLUTION_HEIGHT: int = 1201

    # F1202: Video frame rate (frames per second)
    FRAMERATE: int = 1202

    # F1203: Media bitrate in kilobits per second
    BITRATE: int = 1203

    # F1204: Media codec (H.264, H.265, VP9, AV1, etc.)
    CODEC: int = 1204

    # F1205: Media duration in seconds
    MEDIA_DURATION: int = 1205

    # F1206: Audio sample rate in Hertz
    SAMPLE_RATE: int = 1206

    # F1207: Number of audio channels (1=mono, 2=stereo)
    AUDIO_CHANNELS: int = 1207

    # F1208: Video aspect ratio (16:9, 4:3, etc.)
    ASPECT_RATIO: int = 1208

    # F1300: Blockchain block number/height
    BLOCK_NUMBER: int = 1300

    # F1301: Blockchain block hash
    BLOCK_HASH: int = 1301

    # F1302: Blockchain transaction hash
    TRANSACTION_HASH: int = 1302

    # F1303: Sender wallet address
    FROM_ADDRESS: int = 1303

    # F1304: Recipient wallet address
    TO_ADDRESS: int = 1304

    # F1305: Transaction gas limit
    GAS_LIMIT: int = 1305

    # F1306: Gas price in smallest unit (e.g., wei)
    GAS_PRICE: int = 1306

    # F1307: Account nonce or transaction counter
    NONCE: int = 1307

    # F1400: ISO 4217 currency code (usd, eur, jpy)
    CURRENCY: int = 1400

    # F1401: Monetary amount in smallest unit (cents, yen)
    AMOUNT: int = 1401

    # F1402: Tax portion of the amount
    TAX_AMOUNT: int = 1402

    # F1403: Discount portion of the amount
    DISCOUNT_AMOUNT: int = 1403

    # F1404: Payment method identifier (card, bank_transfer)
    PAYMENT_METHOD: int = 1404

    # F1405: Transaction status (succeeded, pending, failed)
    TRANSACTION_STATUS: int = 1405

    # F1500: File size in bytes
    FILE_SIZE: int = 1500

    # F1501: IANA media type (same as content_type)
    MIME_TYPE: int = 1501

    # F1502: File name extension (without dot)
    FILE_EXTENSION: int = 1502

    # F1503: MD5 checksum hex string
    CHECKSUM_MD5: int = 1503

    # F1504: SHA-256 checksum hex string
    CHECKSUM_SHA256: int = 1504

    # F1505: Last modification timestamp
    LAST_MODIFIED: int = 1505

    # F1600: Communicative act (request, inform, propose)
    PERFORMATIVE: int = 1600

    # F1601: Ontology name used in content
    ONTOLOGY: int = 1601

    # F1602: Interaction protocol (contract-net, auction)
    PROTOCOL: int = 1602

    # F1603: Content language (fipa-sl, kilo, json)
    LANGUAGE: int = 1603

    # F1604: Identifier for reply correlation
    REPLY_WITH: int = 1604

    # F1605: Reference to original message identifier
    IN_REPLY_TO: int = 1605

    # F1621: Device firmware version string
    FIRMWARE_VERSION: int = 1621

    # F1622: Device serial number
    SERIAL_NUMBER: int = 1622

    # F1623: Device model identifier
    DEVICE_MODEL: int = 1623

    # F1641: OPC UA Node Identifier
    NODE_ID: int = 1641

    # F1642: Non-localized browse name
    BROWSE_NAME: int = 1642

    # F1643: Localized display name
    DISPLAY_NAME: int = 1643

    # F1644: Node class (Object, Variable, Method)
    NODE_CLASS: int = 1644

    # F1645: Namespace index for node identifier
    NAMESPACE_INDEX: int = 1645


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
    80: 'trace_id',
    81: 'span_id',
    82: 'parent_span_id',
    83: 'trace_flags',
    84: 'service_name',
    85: 'service_version',
    100: 'timestamp_ns',
    101: 'duration_ms',
    102: 'duration_ns',
    103: 'start_time',
    104: 'end_time',
    105: 'created_at',
    106: 'updated_at',
    120: 'event_type',
    121: 'event_source',
    122: 'correlation_id',
    123: 'request_id',
    124: 'transaction_id',
    125: 'tenant_id',
    126: 'organization_id',
    127: 'device_id',
    128: 'stream_id',
    129: 'channel_id',
    130: 'auth_token',
    131: 'refresh_token',
    132: 'token_expiry',
    133: 'scopes',
    134: 'permissions',
    135: 'api_key',
    256: 'position',
    257: 'rotation',
    258: 'velocity',
    259: 'acceleration',
    260: 'quaternion',
    261: 'bounding_box',
    280: 'latitude',
    281: 'longitude',
    282: 'altitude',
    283: 'heading',
    284: 'ground_speed',
    285: 'position_accuracy',
    286: 'geo_hash',
    300: 'url',
    301: 'hostname',
    302: 'port',
    303: 'ip_address',
    304: 'http_method',
    305: 'http_status_code',
    306: 'user_agent',
    307: 'content_type',
    308: 'content_length',
    309: 'encoding',
    512: 'embedding',
    513: 'embedding_model',
    514: 'embedding_dim',
    520: 'angular_velocity',
    521: 'linear_acceleration',
    522: 'magnetic_field',
    523: 'orientation',
    524: 'joint_positions',
    525: 'joint_velocities',
    526: 'joint_efforts',
    527: 'joint_names',
    528: 'waypoints',
    529: 'point_cloud',
    530: 'twist',
    768: 'temperature',
    769: 'humidity',
    770: 'pressure',
    771: 'battery_level',
    772: 'luminosity',
    773: 'noise_level',
    774: 'co2_level',
    775: 'pm25',
    776: 'pm10',
    777: 'voc',
    778: 'uv_index',
    779: 'wind_speed',
    780: 'wind_direction',
    781: 'rainfall',
    782: 'soil_moisture',
    783: 'ph_level',
    784: 'signal_strength',
    1024: 'message_kind',
    1025: 'ttl',
    1026: 'qos_priority',
    1027: 'retry_count',
    1028: 'delivery_status',
    1029: 'ack_required',
    1030: 'payload_schema',
    1031: 'compression',
    1032: 'encryption',
    1040: 'topic',
    1041: 'partition_id',
    1042: 'offset',
    1043: 'consumer_group',
    1044: 'message_key',
    1045: 'broker_id',
    1100: 'model_id',
    1101: 'model_version',
    1102: 'inference_time',
    1103: 'predictions',
    1104: 'predicted_class',
    1105: 'class_labels',
    1106: 'features',
    1107: 'attention_weights',
    1108: 'token_ids',
    1109: 'input_tokens',
    1110: 'output_tokens',
    1111: 'prompt',
    1112: 'completion',
    1113: 'llm_temperature',
    1114: 'max_tokens',
    1200: 'resolution_width',
    1201: 'resolution_height',
    1202: 'framerate',
    1203: 'bitrate',
    1204: 'codec',
    1205: 'media_duration',
    1206: 'sample_rate',
    1207: 'audio_channels',
    1208: 'aspect_ratio',
    1300: 'block_number',
    1301: 'block_hash',
    1302: 'transaction_hash',
    1303: 'from_address',
    1304: 'to_address',
    1305: 'gas_limit',
    1306: 'gas_price',
    1307: 'nonce',
    1400: 'currency',
    1401: 'amount',
    1402: 'tax_amount',
    1403: 'discount_amount',
    1404: 'payment_method',
    1405: 'transaction_status',
    1500: 'file_size',
    1501: 'mime_type',
    1502: 'file_extension',
    1503: 'checksum_md5',
    1504: 'checksum_sha256',
    1505: 'last_modified',
    1600: 'performative',
    1601: 'ontology',
    1602: 'protocol',
    1603: 'language',
    1604: 'reply_with',
    1605: 'in_reply_to',
    1621: 'firmware_version',
    1622: 'serial_number',
    1623: 'device_model',
    1641: 'node_id',
    1642: 'browse_name',
    1643: 'display_name',
    1644: 'node_class',
    1645: 'namespace_index',
}
