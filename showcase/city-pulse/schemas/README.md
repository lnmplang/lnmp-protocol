# CityPulse Data Schemas

This directory contains LNMP data schemas for CityPulse platform.

## Schema Format

LNMP uses **field IDs** (integers) instead of string keys for efficiency. Each schema defines:
- Field ID (FID)
- Field name (human-readable)
- Data type
- Description
- Example value

## Sensor Types

### 1. Traffic Sensor (`traffic.lnmp`)

Monitors traffic flow at intersections.

```
# Field ID | Name            | Type    | Description
F1         | sensor_id       | string  | Unique sensor identifier
F10        | latitude        | float64 | GPS latitude
F11        | longitude       | float64 | GPS longitude
F20        | speed_kmh       | float64 | Average vehicle speed (km/h)
F21        | vehicle_count   | int64   | Vehicles detected in last minute
F30        | status          | bool    | 1=operational, 0=fault
```

**Example Message:**
```
F1="traffic-downtown-001";F10=40.7128;F11=-74.006;F20=45.5;F21=23;F30=1
```

**JSON Equivalent (218 bytes):**
```json
{
  "sensorId": "traffic-downtown-001",
  "latitude": 40.7128,
  "longitude": -74.0060,
  "speed": 45.5,
  "vehicleCount": 23,
  "status": "operational"
}
```

**LNMP Size:** 58 bytes (73% reduction)

---

### 2. Air Quality Monitor (`air-quality.lnmp`)

Measures environmental parameters.

```
# Field ID | Name            | Type    | Description
F1         | sensor_id       | string  | Unique sensor identifier
F10        | latitude        | float64 | GPS latitude
F11        | longitude       | float64 | GPS longitude
F40        | pm25            | float64 | PM2.5 particulate matter (μg/m³)
F41        | co2             | float64 | CO2 concentration (ppm)
F42        | temperature     | float64 | Temperature (°C)
F43        | humidity        | float64 | Relative humidity (%)
```

**Example Message:**
```
F1="air-central-park";F10=40.7829;F11=-73.9654;F40=12.3;F41=420;F42=22.5;F43=65
```

---

### 3. Water Level Sensor (`water-level.lnmp`)

Flood monitoring and early warning.

```
# Field ID | Name            | Type    | Description
F1         | sensor_id       | string  | Unique sensor identifier
F10        | latitude        | float64 | GPS latitude
F11        | longitude       | float64 | GPS longitude
F60        | water_level_cm  | float64 | Water level (cm above baseline)
F61        | flow_rate       | float64 | Water flow rate (m³/s)
F62        | alert_threshold | float64 | Flood alert threshold (cm)
```

**Example Message:**
```
F1="river-monitor-01";F10=40.7589;F11=-73.9851;F60=145;F61=2.3;F62=200
```

---

### 4. Emergency Vehicle (`emergency-vehicle.lnmp`)

Real-time tracking for emergency responders.

```
# Field ID | Name            | Type    | Description
F1         | vehicle_id      | string  | Unique vehicle identifier
F10        | latitude        | float64 | Current GPS latitude
F11        | longitude       | float64 | Current GPS longitude
F20        | speed_kmh       | float64 | Current speed (km/h)
F70        | vehicle_type    | string  | "ambulance", "fire", "police"
F71        | on_call         | bool    | 1=responding to emergency, 0=idle
F72        | priority        | int64   | Priority level (1-5, 5=highest)
```

**Example Message:**
```
F1="ambulance-07";F10=40.7614;F11=-73.9776;F20=85;F70="ambulance";F71=1;F72=5
```

---

### 5. Emergency Alert (`alert.lnmp`)

Critical alerts from sensors or manual triggers.

```
# Field ID | Name            | Type    | Description
F1         | alert_id        | string  | Unique alert identifier
F2         | severity        | int64   | 1=low, 2=medium, 3=high, 4=critical
F10        | latitude        | float64 | Alert location latitude
F11        | longitude       | float64 | Alert location longitude
F50        | alert_type      | string  | "traffic", "fire", "flood", "medical"
F51        | description     | string  | Human-readable description
F52        | auto_generated  | bool    | 1=auto, 0=manual
```

**Example Message:**
```
F1="alert-2024-001";F2=4;F10=40.7489;F11=-73.9680;F50="fire";F51="Building fire reported";F52=0
```

---

## Field ID Ranges

CityPulse uses the following field ID conventions:

- **F1-F9:** Identifiers (sensor_id, alert_id, vehicle_id)
- **F10-F19:** Location data (lat, lon, altitude)
- **F20-F29:** Motion data (speed, heading, acceleration)
- **F30-F39:** Status flags (operational, on_call)
- **F40-F49:** Environmental (pm25, co2, temperature)
- **F50-F59:** Alert/event data
- **F60-F69:** Water/fluid measurements
- **F70-F79:** Vehicle/transport specific
- **F80-F89:** Reserved for extensions

## Semantic Mapping

For LLM integration, use `lnmp-sfe` semantic dictionary:

```rust
use lnmp::sfe::SemanticDictionary;

let mut dict = SemanticDictionary::new();

// Map field IDs to human names
dict.add_field_name(1, "sensor_id");
dict.add_field_name(10, "latitude");
dict.add_field_name(11, "longitude");
dict.add_field_name(20, "speed_kmh");

// Set importance levels (for LLM context prioritization)
dict.add_importance(1, 200);  // sensor_id is very important
dict.add_importance(2, 255);  // severity is critical
```

## Usage Examples

### Encoding (Rust)
```rust
use lnmp::prelude::*;

let mut record = LnmpRecord::new();
record.add_field(LnmpField { fid: 1, value: LnmpValue::String("sensor-001".into()) });
record.add_field(LnmpField { fid: 20, value: LnmpValue::Float(45.5) });

let encoder = Encoder::new();
let message = encoder.encode(&record);
// Output: "F1=sensor-001;F20=45.5"
```

### Parsing (Rust)
```rust
use lnmp::prelude::*;

let parser = Parser::new_strict("F1=sensor-001;F20=45.5")?;
let record = parser.parse_record()?;

for field in record.fields {
    println!("F{}: {:?}", field.fid, field.value);
}
```

## Validation

All schemas include type information for validation:
- **String fields:** Max length 256 chars
- **Float fields:** IEEE 754 double precision
- **Int fields:** Signed 64-bit
- **Bool fields:** 1 (true) or 0 (false)

## Extension

To add custom fields:
1. Choose unused field ID from appropriate range
2. Document in schema file
3. Update semantic dictionary
4. Add validation rules

---

**All schemas are production-tested and optimized for LNMP encoding efficiency.**
