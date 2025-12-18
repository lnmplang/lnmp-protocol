// Package fid provides LNMP Field ID constants.
//
// Auto-generated from registry/fids.yaml v1.2.0
// Generated: 2025-12-18T22:38:44.883890
//
// DO NOT EDIT MANUALLY
package fid

// Fid is the Field ID type
type Fid uint16

// LNMP Official Field IDs
const (
	// FidEntityId - F1: Unique entity identifier within a context
	FidEntityId Fid = 1

	// FidTimestamp - F2: Unix timestamp in milliseconds (UTC)
	FidTimestamp Fid = 2

	// FidVersion - F3: Schema or data version number
	FidVersion Fid = 3

	// FidSequence - F4: Monotonic sequence number for ordering
	FidSequence Fid = 4

	// FidSource - F5: Origin identifier (service, device, node)
	FidSource Fid = 5

	// FidIsActive - F7: Active/inactive boolean flag
	FidIsActive Fid = 7

	// FidIsValid - F8: Validity boolean flag
	FidIsValid Fid = 8

	// FidUserId - F12: User identifier
	FidUserId Fid = 12

	// FidSessionId - F13: Session identifier
	FidSessionId Fid = 13

	// FidName - F20: Human-readable name
	FidName Fid = 20

	// FidLabel - F21: Short label or tag
	FidLabel Fid = 21

	// FidDescription - F22: Long-form description text
	FidDescription Fid = 22

	// FidRoles - F23: List of role identifiers
	FidRoles Fid = 23

	// FidTags - F24: List of tags for categorization
	FidTags Fid = 24

	// FidCount - F30: Generic count value
	FidCount Fid = 30

	// FidIndex - F31: Zero-based index
	FidIndex Fid = 31

	// FidPriority - F32: Priority level (0 = lowest)
	FidPriority Fid = 32

	// FidValue - F40: Generic floating-point value
	FidValue Fid = 40

	// FidScore - F41: Score or rating (typically 0.0-1.0)
	FidScore Fid = 41

	// FidConfidence - F42: Confidence level (0.0-1.0)
	FidConfidence Fid = 42

	// FidStatus - F50: Status string (pending, active, completed, etc.)
	FidStatus Fid = 50

	// FidErrorCode - F51: Error code (0 = no error)
	FidErrorCode Fid = 51

	// FidErrorMessage - F52: Human-readable error message
	FidErrorMessage Fid = 52

	// FidIntValues - F60: Generic integer array for numeric data
	FidIntValues Fid = 60

	// FidFloatValues - F61: Generic float array for measurement data
	FidFloatValues Fid = 61

	// FidBoolFlags - F62: Generic boolean array for flag sets
	FidBoolFlags Fid = 62

	// FidNestedData - F70: Generic nested record container
	FidNestedData Fid = 70

	// FidRecordList - F71: Array of nested records
	FidRecordList Fid = 71

	// FidTraceId - F80: Distributed trace identifier (W3C Trace Context compatible)
	FidTraceId Fid = 80

	// FidSpanId - F81: Span identifier within a trace
	FidSpanId Fid = 81

	// FidParentSpanId - F82: Parent span identifier for trace hierarchy
	FidParentSpanId Fid = 82

	// FidTraceFlags - F83: Trace flags (sampled, random, etc.)
	FidTraceFlags Fid = 83

	// FidServiceName - F84: Originating service name (OpenTelemetry convention)
	FidServiceName Fid = 84

	// FidServiceVersion - F85: Originating service version
	FidServiceVersion Fid = 85

	// FidTimestampNs - F100: Unix timestamp in nanoseconds (high-precision)
	FidTimestampNs Fid = 100

	// FidDurationMs - F101: Duration/elapsed time in milliseconds
	FidDurationMs Fid = 101

	// FidDurationNs - F102: Duration/elapsed time in nanoseconds
	FidDurationNs Fid = 102

	// FidStartTime - F103: Start timestamp in milliseconds
	FidStartTime Fid = 103

	// FidEndTime - F104: End timestamp in milliseconds
	FidEndTime Fid = 104

	// FidCreatedAt - F105: Creation timestamp in milliseconds
	FidCreatedAt Fid = 105

	// FidUpdatedAt - F106: Last update timestamp in milliseconds
	FidUpdatedAt Fid = 106

	// FidEventType - F120: Event type classification (CloudEvents type)
	FidEventType Fid = 120

	// FidEventSource - F121: Event origin URI (CloudEvents source)
	FidEventSource Fid = 121

	// FidCorrelationId - F122: Request correlation identifier for distributed systems
	FidCorrelationId Fid = 122

	// FidRequestId - F123: Unique request identifier
	FidRequestId Fid = 123

	// FidTransactionId - F124: Transaction identifier for business processes
	FidTransactionId Fid = 124

	// FidTenantId - F125: Multi-tenancy identifier
	FidTenantId Fid = 125

	// FidOrganizationId - F126: Organization/workspace identifier
	FidOrganizationId Fid = 126

	// FidDeviceId - F127: Physical device identifier
	FidDeviceId Fid = 127

	// FidStreamId - F128: Data stream identifier
	FidStreamId Fid = 128

	// FidChannelId - F129: Communication channel identifier
	FidChannelId Fid = 129

	// FidAuthToken - F130: Authentication token (JWT, OAuth, etc.)
	FidAuthToken Fid = 130

	// FidRefreshToken - F131: Token refresh credential
	FidRefreshToken Fid = 131

	// FidTokenExpiry - F132: Token expiration timestamp in milliseconds
	FidTokenExpiry Fid = 132

	// FidScopes - F133: OAuth2 scopes or permission scopes
	FidScopes Fid = 133

	// FidPermissions - F134: Access permission identifiers
	FidPermissions Fid = 134

	// FidApiKey - F135: API key for service authentication
	FidApiKey Fid = 135

	// FidPosition - F256: [x, y, z] position coordinates in meters
	FidPosition Fid = 256

	// FidRotation - F257: [roll, pitch, yaw] Euler angles in radians
	FidRotation Fid = 257

	// FidVelocity - F258: [vx, vy, vz] linear velocity
	FidVelocity Fid = 258

	// FidAcceleration - F259: [ax, ay, az] linear acceleration
	FidAcceleration Fid = 259

	// FidQuaternion - F260: [w, x, y, z] rotation quaternion
	FidQuaternion Fid = 260

	// FidBoundingBox - F261: [min_x, min_y, min_z, max_x, max_y, max_z]
	FidBoundingBox Fid = 261

	// FidLatitude - F280: WGS84 latitude (-90 to 90 degrees)
	FidLatitude Fid = 280

	// FidLongitude - F281: WGS84 longitude (-180 to 180 degrees)
	FidLongitude Fid = 281

	// FidAltitude - F282: Altitude above sea level in meters
	FidAltitude Fid = 282

	// FidHeading - F283: Compass heading (0-360 degrees, 0=North)
	FidHeading Fid = 283

	// FidGroundSpeed - F284: Ground speed in meters per second
	FidGroundSpeed Fid = 284

	// FidPositionAccuracy - F285: Position accuracy radius in meters
	FidPositionAccuracy Fid = 285

	// FidGeoHash - F286: GeoHash encoded location string
	FidGeoHash Fid = 286

	// FidUrl - F300: Full URL/URI
	FidUrl Fid = 300

	// FidHostname - F301: Host name or IP address
	FidHostname Fid = 301

	// FidPort - F302: Network port number (0-65535)
	FidPort Fid = 302

	// FidIpAddress - F303: IPv4 or IPv6 address
	FidIpAddress Fid = 303

	// FidHttpMethod - F304: HTTP method (GET, POST, PUT, DELETE, etc.)
	FidHttpMethod Fid = 304

	// FidHttpStatusCode - F305: HTTP response status code (200, 404, 500, etc.)
	FidHttpStatusCode Fid = 305

	// FidUserAgent - F306: HTTP User-Agent header value
	FidUserAgent Fid = 306

	// FidContentType - F307: MIME content type (application/json, etc.)
	FidContentType Fid = 307

	// FidContentLength - F308: Content size in bytes
	FidContentLength Fid = 308

	// FidEncoding - F309: Content encoding (utf-8, gzip, etc.)
	FidEncoding Fid = 309

	// FidEmbedding - F512: Vector embedding (variable dimension)
	FidEmbedding Fid = 512

	// FidEmbeddingModel - F513: Embedding model identifier
	FidEmbeddingModel Fid = 513

	// FidEmbeddingDim - F514: Embedding dimension size
	FidEmbeddingDim Fid = 514

	// FidAngularVelocity - F520: [wx, wy, wz] angular velocity
	FidAngularVelocity Fid = 520

	// FidLinearAcceleration - F521: [ax, ay, az] IMU linear acceleration
	FidLinearAcceleration Fid = 521

	// FidMagneticField - F522: [mx, my, mz] magnetic field in Tesla
	FidMagneticField Fid = 522

	// FidOrientation - F523: [w, x, y, z] orientation quaternion from IMU
	FidOrientation Fid = 523

	// FidJointPositions - F524: Robot joint positions in radians
	FidJointPositions Fid = 524

	// FidJointVelocities - F525: Robot joint velocities
	FidJointVelocities Fid = 525

	// FidJointEfforts - F526: Robot joint torques/forces
	FidJointEfforts Fid = 526

	// FidJointNames - F527: Robot joint names
	FidJointNames Fid = 527

	// FidWaypoints - F528: Navigation waypoints as records
	FidWaypoints Fid = 528

	// FidPointCloud - F529: Flattened 3D point cloud [x,y,z,x,y,z,...]
	FidPointCloud Fid = 529

	// FidTwist - F530: [vx,vy,vz,wx,wy,wz] linear+angular velocity (ROS Twist)
	FidTwist Fid = 530

	// FidTemperature - F768: Temperature in Celsius
	FidTemperature Fid = 768

	// FidHumidity - F769: Relative humidity percentage
	FidHumidity Fid = 769

	// FidPressure - F770: Pressure in Pascals
	FidPressure Fid = 770

	// FidBatteryLevel - F771: Battery charge percentage
	FidBatteryLevel Fid = 771

	// FidLuminosity - F772: Light level in lux
	FidLuminosity Fid = 772

	// FidNoiseLevel - F773: Sound level in decibels
	FidNoiseLevel Fid = 773

	// FidCo2Level - F774: CO2 concentration in parts per million
	FidCo2Level Fid = 774

	// FidPm25 - F775: PM2.5 particulate matter concentration
	FidPm25 Fid = 775

	// FidPm10 - F776: PM10 particulate matter concentration
	FidPm10 Fid = 776

	// FidVoc - F777: Volatile organic compounds level
	FidVoc Fid = 777

	// FidUvIndex - F778: UV radiation index (0-11+)
	FidUvIndex Fid = 778

	// FidWindSpeed - F779: Wind speed in meters per second
	FidWindSpeed Fid = 779

	// FidWindDirection - F780: Wind direction (0-360 degrees, 0=North)
	FidWindDirection Fid = 780

	// FidRainfall - F781: Rainfall accumulation in millimeters
	FidRainfall Fid = 781

	// FidSoilMoisture - F782: Soil moisture percentage
	FidSoilMoisture Fid = 782

	// FidPhLevel - F783: pH level (0-14 scale)
	FidPhLevel Fid = 783

	// FidSignalStrength - F784: Signal strength (RSSI) in dBm
	FidSignalStrength Fid = 784

	// FidMessageKind - F1024: Message classification (Event, State, Command, Query, Alert)
	FidMessageKind Fid = 1024

	// FidTtl - F1025: Time-to-live in milliseconds
	FidTtl Fid = 1025

	// FidQosPriority - F1026: QoS priority level (0-255)
	FidQosPriority Fid = 1026

	// FidRetryCount - F1027: Number of delivery retries
	FidRetryCount Fid = 1027

	// FidDeliveryStatus - F1028: Message delivery status (pending, delivered, failed)
	FidDeliveryStatus Fid = 1028

	// FidAckRequired - F1029: Whether acknowledgment is required
	FidAckRequired Fid = 1029

	// FidPayloadSchema - F1030: Schema identifier for payload (CloudEvents dataschema)
	FidPayloadSchema Fid = 1030

	// FidCompression - F1031: Compression algorithm (none, gzip, lz4, zstd)
	FidCompression Fid = 1031

	// FidEncryption - F1032: Encryption algorithm (none, aes256, chacha20)
	FidEncryption Fid = 1032

	// FidTopic - F1040: Message queue topic name (Kafka, RabbitMQ, etc.)
	FidTopic Fid = 1040

	// FidPartitionId - F1041: Kafka partition identifier
	FidPartitionId Fid = 1041

	// FidOffset - F1042: Message offset within partition
	FidOffset Fid = 1042

	// FidConsumerGroup - F1043: Consumer group identifier
	FidConsumerGroup Fid = 1043

	// FidMessageKey - F1044: Message partitioning key
	FidMessageKey Fid = 1044

	// FidBrokerId - F1045: Message broker identifier
	FidBrokerId Fid = 1045

	// FidModelId - F1100: ML model identifier
	FidModelId Fid = 1100

	// FidModelVersion - F1101: ML model version
	FidModelVersion Fid = 1101

	// FidInferenceTime - F1102: Model inference time in milliseconds
	FidInferenceTime Fid = 1102

	// FidPredictions - F1103: Model prediction probabilities
	FidPredictions Fid = 1103

	// FidPredictedClass - F1104: Predicted class index
	FidPredictedClass Fid = 1104

	// FidClassLabels - F1105: Class label names
	FidClassLabels Fid = 1105

	// FidFeatures - F1106: Input feature vector
	FidFeatures Fid = 1106

	// FidAttentionWeights - F1107: Attention/importance weights
	FidAttentionWeights Fid = 1107

	// FidTokenIds - F1108: Tokenized input IDs
	FidTokenIds Fid = 1108

	// FidInputTokens - F1109: Input tokens for NLP
	FidInputTokens Fid = 1109

	// FidOutputTokens - F1110: Output tokens for NLP
	FidOutputTokens Fid = 1110

	// FidPrompt - F1111: LLM prompt text
	FidPrompt Fid = 1111

	// FidCompletion - F1112: LLM completion text
	FidCompletion Fid = 1112

	// FidLlmTemperature - F1113: LLM sampling temperature (0.0-2.0)
	FidLlmTemperature Fid = 1113

	// FidMaxTokens - F1114: Maximum output tokens for LLM
	FidMaxTokens Fid = 1114

	// FidResolutionWidth - F1200: Video/image width in pixels
	FidResolutionWidth Fid = 1200

	// FidResolutionHeight - F1201: Video/image height in pixels
	FidResolutionHeight Fid = 1201

	// FidFramerate - F1202: Video frame rate (frames per second)
	FidFramerate Fid = 1202

	// FidBitrate - F1203: Media bitrate in kilobits per second
	FidBitrate Fid = 1203

	// FidCodec - F1204: Media codec (H.264, H.265, VP9, AV1, etc.)
	FidCodec Fid = 1204

	// FidMediaDuration - F1205: Media duration in seconds
	FidMediaDuration Fid = 1205

	// FidSampleRate - F1206: Audio sample rate in Hertz
	FidSampleRate Fid = 1206

	// FidAudioChannels - F1207: Number of audio channels (1=mono, 2=stereo)
	FidAudioChannels Fid = 1207

	// FidAspectRatio - F1208: Video aspect ratio (16:9, 4:3, etc.)
	FidAspectRatio Fid = 1208

	// FidBlockNumber - F1300: Blockchain block number/height
	FidBlockNumber Fid = 1300

	// FidBlockHash - F1301: Blockchain block hash
	FidBlockHash Fid = 1301

	// FidTransactionHash - F1302: Blockchain transaction hash
	FidTransactionHash Fid = 1302

	// FidFromAddress - F1303: Sender wallet address
	FidFromAddress Fid = 1303

	// FidToAddress - F1304: Recipient wallet address
	FidToAddress Fid = 1304

	// FidGasLimit - F1305: Transaction gas limit
	FidGasLimit Fid = 1305

	// FidGasPrice - F1306: Gas price in smallest unit (e.g., wei)
	FidGasPrice Fid = 1306

	// FidNonce - F1307: Account nonce or transaction counter
	FidNonce Fid = 1307

	// FidCurrency - F1400: ISO 4217 currency code (usd, eur, jpy)
	FidCurrency Fid = 1400

	// FidAmount - F1401: Monetary amount in smallest unit (cents, yen)
	FidAmount Fid = 1401

	// FidTaxAmount - F1402: Tax portion of the amount
	FidTaxAmount Fid = 1402

	// FidDiscountAmount - F1403: Discount portion of the amount
	FidDiscountAmount Fid = 1403

	// FidPaymentMethod - F1404: Payment method identifier (card, bank_transfer)
	FidPaymentMethod Fid = 1404

	// FidTransactionStatus - F1405: Transaction status (succeeded, pending, failed)
	FidTransactionStatus Fid = 1405

	// FidFileSize - F1500: File size in bytes
	FidFileSize Fid = 1500

	// FidMimeType - F1501: IANA media type (same as content_type)
	FidMimeType Fid = 1501

	// FidFileExtension - F1502: File name extension (without dot)
	FidFileExtension Fid = 1502

	// FidChecksumMd5 - F1503: MD5 checksum hex string
	FidChecksumMd5 Fid = 1503

	// FidChecksumSha256 - F1504: SHA-256 checksum hex string
	FidChecksumSha256 Fid = 1504

	// FidLastModified - F1505: Last modification timestamp
	FidLastModified Fid = 1505

	// FidPerformative - F1600: Communicative act (request, inform, propose)
	FidPerformative Fid = 1600

	// FidOntology - F1601: Ontology name used in content
	FidOntology Fid = 1601

	// FidProtocol - F1602: Interaction protocol (contract-net, auction)
	FidProtocol Fid = 1602

	// FidLanguage - F1603: Content language (fipa-sl, kilo, json)
	FidLanguage Fid = 1603

	// FidReplyWith - F1604: Identifier for reply correlation
	FidReplyWith Fid = 1604

	// FidInReplyTo - F1605: Reference to original message identifier
	FidInReplyTo Fid = 1605

	// FidFirmwareVersion - F1621: Device firmware version string
	FidFirmwareVersion Fid = 1621

	// FidSerialNumber - F1622: Device serial number
	FidSerialNumber Fid = 1622

	// FidDeviceModel - F1623: Device model identifier
	FidDeviceModel Fid = 1623

	// FidNodeId - F1641: OPC UA Node Identifier
	FidNodeId Fid = 1641

	// FidBrowseName - F1642: Non-localized browse name
	FidBrowseName Fid = 1642

	// FidDisplayName - F1643: Localized display name
	FidDisplayName Fid = 1643

	// FidNodeClass - F1644: Node class (Object, Variable, Method)
	FidNodeClass Fid = 1644

	// FidNamespaceIndex - F1645: Namespace index for node identifier
	FidNamespaceIndex Fid = 1645

)

// FidNames maps FID numbers to names
var FidNames = map[Fid]string{
	FidEntityId: "entity_id",
	FidTimestamp: "timestamp",
	FidVersion: "version",
	FidSequence: "sequence",
	FidSource: "source",
	FidIsActive: "is_active",
	FidIsValid: "is_valid",
	FidUserId: "user_id",
	FidSessionId: "session_id",
	FidName: "name",
	FidLabel: "label",
	FidDescription: "description",
	FidRoles: "roles",
	FidTags: "tags",
	FidCount: "count",
	FidIndex: "index",
	FidPriority: "priority",
	FidValue: "value",
	FidScore: "score",
	FidConfidence: "confidence",
	FidStatus: "status",
	FidErrorCode: "error_code",
	FidErrorMessage: "error_message",
	FidIntValues: "int_values",
	FidFloatValues: "float_values",
	FidBoolFlags: "bool_flags",
	FidNestedData: "nested_data",
	FidRecordList: "record_list",
	FidTraceId: "trace_id",
	FidSpanId: "span_id",
	FidParentSpanId: "parent_span_id",
	FidTraceFlags: "trace_flags",
	FidServiceName: "service_name",
	FidServiceVersion: "service_version",
	FidTimestampNs: "timestamp_ns",
	FidDurationMs: "duration_ms",
	FidDurationNs: "duration_ns",
	FidStartTime: "start_time",
	FidEndTime: "end_time",
	FidCreatedAt: "created_at",
	FidUpdatedAt: "updated_at",
	FidEventType: "event_type",
	FidEventSource: "event_source",
	FidCorrelationId: "correlation_id",
	FidRequestId: "request_id",
	FidTransactionId: "transaction_id",
	FidTenantId: "tenant_id",
	FidOrganizationId: "organization_id",
	FidDeviceId: "device_id",
	FidStreamId: "stream_id",
	FidChannelId: "channel_id",
	FidAuthToken: "auth_token",
	FidRefreshToken: "refresh_token",
	FidTokenExpiry: "token_expiry",
	FidScopes: "scopes",
	FidPermissions: "permissions",
	FidApiKey: "api_key",
	FidPosition: "position",
	FidRotation: "rotation",
	FidVelocity: "velocity",
	FidAcceleration: "acceleration",
	FidQuaternion: "quaternion",
	FidBoundingBox: "bounding_box",
	FidLatitude: "latitude",
	FidLongitude: "longitude",
	FidAltitude: "altitude",
	FidHeading: "heading",
	FidGroundSpeed: "ground_speed",
	FidPositionAccuracy: "position_accuracy",
	FidGeoHash: "geo_hash",
	FidUrl: "url",
	FidHostname: "hostname",
	FidPort: "port",
	FidIpAddress: "ip_address",
	FidHttpMethod: "http_method",
	FidHttpStatusCode: "http_status_code",
	FidUserAgent: "user_agent",
	FidContentType: "content_type",
	FidContentLength: "content_length",
	FidEncoding: "encoding",
	FidEmbedding: "embedding",
	FidEmbeddingModel: "embedding_model",
	FidEmbeddingDim: "embedding_dim",
	FidAngularVelocity: "angular_velocity",
	FidLinearAcceleration: "linear_acceleration",
	FidMagneticField: "magnetic_field",
	FidOrientation: "orientation",
	FidJointPositions: "joint_positions",
	FidJointVelocities: "joint_velocities",
	FidJointEfforts: "joint_efforts",
	FidJointNames: "joint_names",
	FidWaypoints: "waypoints",
	FidPointCloud: "point_cloud",
	FidTwist: "twist",
	FidTemperature: "temperature",
	FidHumidity: "humidity",
	FidPressure: "pressure",
	FidBatteryLevel: "battery_level",
	FidLuminosity: "luminosity",
	FidNoiseLevel: "noise_level",
	FidCo2Level: "co2_level",
	FidPm25: "pm25",
	FidPm10: "pm10",
	FidVoc: "voc",
	FidUvIndex: "uv_index",
	FidWindSpeed: "wind_speed",
	FidWindDirection: "wind_direction",
	FidRainfall: "rainfall",
	FidSoilMoisture: "soil_moisture",
	FidPhLevel: "ph_level",
	FidSignalStrength: "signal_strength",
	FidMessageKind: "message_kind",
	FidTtl: "ttl",
	FidQosPriority: "qos_priority",
	FidRetryCount: "retry_count",
	FidDeliveryStatus: "delivery_status",
	FidAckRequired: "ack_required",
	FidPayloadSchema: "payload_schema",
	FidCompression: "compression",
	FidEncryption: "encryption",
	FidTopic: "topic",
	FidPartitionId: "partition_id",
	FidOffset: "offset",
	FidConsumerGroup: "consumer_group",
	FidMessageKey: "message_key",
	FidBrokerId: "broker_id",
	FidModelId: "model_id",
	FidModelVersion: "model_version",
	FidInferenceTime: "inference_time",
	FidPredictions: "predictions",
	FidPredictedClass: "predicted_class",
	FidClassLabels: "class_labels",
	FidFeatures: "features",
	FidAttentionWeights: "attention_weights",
	FidTokenIds: "token_ids",
	FidInputTokens: "input_tokens",
	FidOutputTokens: "output_tokens",
	FidPrompt: "prompt",
	FidCompletion: "completion",
	FidLlmTemperature: "llm_temperature",
	FidMaxTokens: "max_tokens",
	FidResolutionWidth: "resolution_width",
	FidResolutionHeight: "resolution_height",
	FidFramerate: "framerate",
	FidBitrate: "bitrate",
	FidCodec: "codec",
	FidMediaDuration: "media_duration",
	FidSampleRate: "sample_rate",
	FidAudioChannels: "audio_channels",
	FidAspectRatio: "aspect_ratio",
	FidBlockNumber: "block_number",
	FidBlockHash: "block_hash",
	FidTransactionHash: "transaction_hash",
	FidFromAddress: "from_address",
	FidToAddress: "to_address",
	FidGasLimit: "gas_limit",
	FidGasPrice: "gas_price",
	FidNonce: "nonce",
	FidCurrency: "currency",
	FidAmount: "amount",
	FidTaxAmount: "tax_amount",
	FidDiscountAmount: "discount_amount",
	FidPaymentMethod: "payment_method",
	FidTransactionStatus: "transaction_status",
	FidFileSize: "file_size",
	FidMimeType: "mime_type",
	FidFileExtension: "file_extension",
	FidChecksumMd5: "checksum_md5",
	FidChecksumSha256: "checksum_sha256",
	FidLastModified: "last_modified",
	FidPerformative: "performative",
	FidOntology: "ontology",
	FidProtocol: "protocol",
	FidLanguage: "language",
	FidReplyWith: "reply_with",
	FidInReplyTo: "in_reply_to",
	FidFirmwareVersion: "firmware_version",
	FidSerialNumber: "serial_number",
	FidDeviceModel: "device_model",
	FidNodeId: "node_id",
	FidBrowseName: "browse_name",
	FidDisplayName: "display_name",
	FidNodeClass: "node_class",
	FidNamespaceIndex: "namespace_index",
}
