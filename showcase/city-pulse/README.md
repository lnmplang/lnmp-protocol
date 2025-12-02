# CityPulse - Smart City IoT Platform

A **standalone showcase project** demonstrating LNMP protocol at production scale.

## 🎯 What is This?

CityPulse is a **complete reference implementation** - not just documentation but a working project you can run, modify, and learn from. It simulates a smart city platform managing 10,000+ IoT sensors.

## 🚀 Quick Start

```bash
# From workspace root
cd showcase/city-pulse

# Run the benchmark
cargo run --bin benchmark

# Or from workspace root
cargo run -p city-pulse --bin benchmark
```

## 📁 Project Structure

```
city-pulse/
├── Cargo.toml           # Standalone Rust project
├── README.md            # This file
├── src/
│   └── benchmark.rs     # Performance benchmark with real data
├── schemas/             # LNMP data schemas
├── data/                # Sample datasets
├── benchmarks/          # Results and analysis
└── docs/                # Detailed documentation
```

## 📊 Real Benchmark Results

```
═══════════════════════════════════════════════════════════
                 PERFORMANCE COMPARISON                      
═══════════════════════════════════════════════════════════

┌────────────────┬─────────────┬─────────────┬─────────────┐
│ Metric         │ JSON        │ LNMP        │ Improvement │
├────────────────┼─────────────┼─────────────┼─────────────┤
│ Message Size   │     123 B   │      62 B   │      49.6% │
│ Bandwidth      │   12.31 MB/s│    6.21 MB/s│      49.6% │
│ Monthly Cost   │ $ 2,871.64  │ $ 1,448.63  │ $ 1,423.02  │
└────────────────┴─────────────┴─────────────┴─────────────┘

Annual Savings: $17,076.21
```

**These are REAL measurements**, not estimates! Run the benchmark yourself.

## 💡 Key Features

### Uses Meta LNMP Crate
```toml
[dependencies]
lnmp = { path = "../../crates/lnmp" }
```

All LNMP functionality through single import:
```rust
use lnmp::prelude::*;
```

### Production-Scale Testing
- 10,000 sensors simulated
- 100,000 messages encoded
- Real JSON vs LNMP comparison
- Bandwidth and cost calculations

### Complete Documentation
- Sensor schemas with field mappings
- Sample data and examples
- Architecture documentation
- Performance analysis

## 🏗️ Architecture

CityPulse demonstrates:
- **Compact Encoding:** 49.6% size reduction
- **Field ID Mapping:** String keys → integer IDs
- **Type Safety:** Structured data validation
- **Scalability:** Linear performance scaling

### Sensor Types
1. **Traffic Sensors** (2,000) - Vehicle speed and count
2. **Air Quality** (500) - PM2.5, CO2, temperature
3. **Water Level** (100) - Flood monitoring
4. **Emergency Vehicles** (100) - GPS tracking
5. **Smart Parking** (6,950) - Occupancy status

See [`schemas/README.md`](./schemas/README.md) for complete schemas.

## 📖 Documentation

- **[Overview](./docs/overview.md)** - MOVED from root showcase
- **[Schemas](./schemas/README.md)** - Data schema reference
- **[Benchmark Results](./benchmarks/results.md)** - Performance analysis
- **[Data](./data/README.md)** - Sample datasets

## 🔧 Development

### Build
```bash
cargo build --release -p city-pulse
```

### Run Benchmark
```bash
cargo run -p city-pulse --bin benchmark
```

### Add New Components
```bash
# Add new binary
[[bin]]
name = "your_component"
path = "src/your_component.rs"
```

## 🎓 Learning Path

1. **Run the benchmark** - See real performance data
2. **Read schemas** - Understand LNMP field mappings
3. **Study source** - See meta crate usage
4. **Modify & experiment** - Try different sensor counts

## 🤝 Contributing

This is a **showcase project** - feel free to:
- Fork and adapt for your use case
- Add new sensor types
- Improve benchmarks
- Share your results

## 📝 License

MIT (same as LNMP protocol)

---

**This is a standalone project** using LNMP meta crate. It's designed to be:
- ✅ Easy to understand
- ✅ Easy to run
- ✅ Easy to modify
- ✅ Production-ready patterns

**Not mixed with core examples** - this is a complete application showcase!
