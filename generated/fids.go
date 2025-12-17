// Package fid provides LNMP Field ID constants.
//
// Auto-generated from registry/fids.yaml v1.0.0
// Generated: 2025-12-17T01:16:15.583380
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

	// FidEmbedding - F512: Vector embedding (variable dimension)
	FidEmbedding Fid = 512

	// FidEmbeddingModel - F513: Embedding model identifier
	FidEmbeddingModel Fid = 513

	// FidEmbeddingDim - F514: Embedding dimension size
	FidEmbeddingDim Fid = 514

	// FidTemperature - F768: Temperature in Celsius
	FidTemperature Fid = 768

	// FidHumidity - F769: Relative humidity percentage
	FidHumidity Fid = 769

	// FidPressure - F770: Pressure in Pascals
	FidPressure Fid = 770

	// FidBatteryLevel - F771: Battery charge percentage
	FidBatteryLevel Fid = 771

	// FidMessageKind - F1024: Message classification (Event, State, Command, Query, Alert)
	FidMessageKind Fid = 1024

	// FidTtl - F1025: Time-to-live in milliseconds
	FidTtl Fid = 1025

	// FidQosPriority - F1026: QoS priority level (0-255)
	FidQosPriority Fid = 1026

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
	FidPosition: "position",
	FidRotation: "rotation",
	FidVelocity: "velocity",
	FidAcceleration: "acceleration",
	FidQuaternion: "quaternion",
	FidBoundingBox: "bounding_box",
	FidEmbedding: "embedding",
	FidEmbeddingModel: "embedding_model",
	FidEmbeddingDim: "embedding_dim",
	FidTemperature: "temperature",
	FidHumidity: "humidity",
	FidPressure: "pressure",
	FidBatteryLevel: "battery_level",
	FidMessageKind: "message_kind",
	FidTtl: "ttl",
	FidQosPriority: "qos_priority",
}
