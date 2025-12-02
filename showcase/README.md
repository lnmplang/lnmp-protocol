# LNMP Protocol - Real-World Showcase

Welcome to the LNMP Protocol showcase! This directory demonstrates how LNMP solves real-world challenges at production scale.

## 🌆 Featured Scenario: CityPulse

**CityPulse** is a smart city IoT platform managing 10,000+ sensors across urban infrastructure.

### The Challenge

A modern city needs to:
- Monitor 10,000+ sensors in real-time
- Process critical alerts within milliseconds
- Minimize bandwidth costs (cloud egress fees)
- Handle diverse data types (telemetry, positions, alerts)
- Maintain trace context for debugging

### The LNMP Solution

| Metric | Traditional (JSON) | LNMP | Improvement |
|--------|-------------------|------|-------------|
| **Message Size** | ~220 bytes | ~60 bytes | **73% smaller** |
| **Bandwidth** | 22 MB/s | 6 MB/s | **73% reduction** |
| **Daily Traffic** | 1.9 TB | 518 GB | **73% savings** |
| **Monthly Cost** | $500 | $135 | **$365 saved** |

### Key Features Demonstrated

- ✅ **Compact Encoding** - 70%+ bandwidth savings
- ✅ **Priority Routing** - Emergency alerts processed first
- ✅ **Context Profiling** - Smart cache invalidation
- ✅ **Delta Encoding** - 85%+ reduction for position updates
- ✅ **Input Sanitization** - XSS/SQL injection protection
- ✅ **Spatial Protocol** - Efficient vehicle tracking
- ✅ **Trace Context** - W3C-compatible distributed tracing

## 📚 Explore the Showcase

### 📖 [CityPulse Scenario](./city-pulse/)
Deep-dive into the smart city use case, architecture, and implementation.

### 🏗️ [Architecture](./architecture/)
System diagrams, data flow, and component design.

### 🎮 [Demos](./demos/) *(coming soon)*
Interactive tools and calculators.

## 🚀 Running Examples

All runnable Rust code is in the **meta crate**:

```bash
# IoT sensor telemetry (based on CityPulse traffic sensors)
cargo run -p lnmp --example iot_sensor_telemetry

# RAG context manager (knowledge base for city operations)
cargo run -p lnmp --example rag_context_manager

# Secure API gateway (public-facing city data API)
cargo run -p lnmp --example secure_api_gateway

# Robot fleet coordinator (emergency vehicle tracking)
cargo run -p lnmp --example robot_fleet_coordinator

# Distributed cache (city-wide data synchronization)
cargo run -p lnmp --example distributed_cache
```

**Source:** `crates/lnmp/examples/*.rs`

## 💡 Why This Matters

### For Developers
- See real-world patterns you can adapt
- Understand when/why to use each feature
- Learn from production-ready code

### For Decision Makers
- Concrete cost savings ($365/month in this scenario)
- Performance improvements (73% bandwidth reduction)
- Scalability proven (10,000+ sensors)

### For System Architects
- Complete architecture patterns
- Integration strategies
- Scaling considerations

## 📖 Documentation

- **Protocol Spec:** See `../spec/` directory
- **API Docs:** Run `cargo doc --open`
- **Feature Examples:** See `../crates/*/examples/`

## 🤝 Next Steps

1. **Explore CityPulse:** Start with [`city-pulse/overview.md`](./city-pulse/overview.md)
2. **Review Architecture:** Check [`architecture/system-design.md`](./architecture/system-design.md)
3. **Run Examples:** Try the Rust examples above
4. **Adapt Patterns:** Use CityPulse as inspiration for your use case

---

**Note:** This showcase focuses on documentation and visualization. For runnable code, see `../crates/lnmp/examples/`.
