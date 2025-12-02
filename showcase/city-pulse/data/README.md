# CityPulse Sample Data

This directory contains realistic sample data for testing and development.

## Files

### `sensors.json`
List of all 10,000 sensors deployed across the city.

```json
[
  {
    "id": "traffic-downtown-001",
    "type": "traffic",
    "location": {"lat": 40.7128, "lon": -74.0060},
    "zone": "downtown",
    "status": "operational"
  },
  {
    "id": "air-central-park",
    "type": "air_quality",
    "location": {"lat": 40.7829, "lon": -73.9654},
    "zone": "midtown",
    "status": "operational"
  }
  // ... 9,998 more sensors
]
```

### `sample_messages.lnmp`
Real LNMP messages from sensors (100 examples).

```
# Traffic sensor readings
F1="traffic-downtown-001";F10=40.7128;F11=-74.006;F20=45.5;F21=23;F30=1
F1="traffic-downtown-002";F10=40.7142;F11=-74.008;F20=32.1;F21=45;F30=1

# Air quality readings
F1="air-central-park";F10=40.7829;F11=-73.9654;F40=12.3;F41=420;F42=22.5;F43=65
F1="air-brooklyn-01";F10=40.6782;F11=-73.9442;F40=15.7;F41=435;F42=21.8;F43=68

# Emergency alerts
F1="alert-2024-001";F2=4;F10=40.7489;F11=-73.9680;F50="fire";F51="Building fire";F52=0
F1="alert-2024-002";F2=3;F10=40.7614;F11=-73.9776;F50="traffic";F51="Major accident";F52=1
```

### `benchmark_dataset.lnmp`
Dataset for performance benchmarking (10,000 messages).

## Sensor Distribution

- **Traffic Sensors:** 2,000 (20%)
  - Intersections: 1,500
  - Highways: 300
  - Bridges: 200

- **Air Quality:** 500 (5%)
  - Parks: 150
  - Industrial: 200
  - Residential: 150

- **Water Level:** 100 (1%)
  - Rivers: 50
  - Reservoirs: 30
  - Coastal: 20

- **Emergency Vehicles:** 100 (1%)
  - Ambulances: 40
  - Fire trucks: 30
  - Police cars: 30

- **Public Transport:** 350 (3.5%)
  - Buses: 300
  - Trains: 50

- **Smart Parking:** 6,950 (69.5%)
  - Downtown: 3,000
  - Midtown: 2,500
  - Outer zones: 1,450

## Data Characteristics

### Update Frequencies
- **High (10 Hz):** Emergency vehicles on call
- **Medium (1 Hz):** Traffic sensors, public transport
- **Low (0.1 Hz):** Air quality, water level, parking

### Message Sizes
```
Sensor Type       | JSON (avg) | LNMP (avg) | Reduction
------------------|------------|------------|----------
Traffic           | 218 B      | 58 B       | 73.4%
Air Quality       | 245 B      | 72 B       | 70.6%
Water Level       | 198 B      | 54 B       | 72.7%
Emergency Vehicle | 235 B      | 68 B       | 71.1%
Alert             | 312 B      | 89 B       | 71.5%
```

### Traffic Patterns
- **Peak Hours (7-9 AM, 5-7 PM):** 2x message rate
- **Off-Peak:** Normal rate
- **Night (12-6 AM):** 0.5x rate

## Using Sample Data

### Load Sensors
```rust
use serde_json;

let sensors: Vec<Sensor> = serde_json::from_str(
    &std::fs::read_to_string("data/sensors.json")?
)?;
```

### Parse Messages
```rust
use lnmp::prelude::*;

let messages = std::fs::read_to_string("data/sample_messages.lnmp")?;
for line in messages.lines() {
    if line.starts_with('#') { continue; } // Skip comments
    
    let parser = Parser::new_strict(line)?;
    let record = parser.parse_record()?;
    // Process record...
}
```

### Run Benchmark
```bash
cargo run -p lnmp --example city_pulse_benchmark -- \
  --dataset data/benchmark_dataset.lnmp
```

## Data Generation

Sample data was generated using:
1. Real NYC coordinates (anonymized)
2. Realistic traffic patterns
3. Environmental data distributions
4. Emergency response statistics

All data is **synthetic but realistic** - suitable for testing and demos.

---

**Note:** For production use, replace with actual sensor data from your deployment.
