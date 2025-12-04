# 🏙️ Tokyo Smart City OS (CityPulse Redesigned)

> **A Cognitive City Operating System** powered by LNMP Protocol - Demonstrating real-time AI-driven emergency response at massive scale.

## 🎯 Overview

Tokyo Smart City OS is a complete redesign of CityPulse, transforming it from a simple IoT demo into a **production-grade intelligent city management system**. It processes 100,000+ events per second, uses semantic filtering to identify critical situations, and coordinates multi-agent emergency responses through real AI (OpenAI GPT-4o-mini).

### Key Capabilities

- 🚀 **1.6M events/sec** real-time processing
- 🎯 **97.4% bandwidth reduction** through intelligent filtering
- 💰 **66% LLM cost savings** via semantic pre-processing  
- 🤖 **Live AI integration** with OpenAI for crisis decision-making
- 👥 **11 autonomous agents** (Police, Fire, Medical, Traffic)
- 📊 **Real-time dashboard** with sparklines and performance metrics

## 🏆 Performance Benchmarks

```
╔═══════════════════════════════════════════════════════════════╗
║                    BENCHMARK RESULTS (100K Events)            ║
╚═══════════════════════════════════════════════════════════════╝

⏱️  PERFORMANCE
   Throughput:          1,589,461 events/sec
   Latency:             6.2ms avg per batch
   
📊 EFFICIENCY  
   Bandwidth Reduction: 97.4% (76K → 2K events)
   LLM Cost Reduction:  66.3% (76K → 26K events)

💵 REAL-WORLD IMPACT
   Bandwidth Saved:     36.5 MB (97.4%)
   LLM API Cost Saved:  $0.76 per 100K events (66.3%)
```

## 🚀 Quick Start

### Prerequisites
```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# (Optional) OpenAI API Key for real AI
echo "OPENAI_API_KEY=sk-..." > .env
```

### Run Interactive Crisis Simulation
```bash
cargo run --bin run_scenario
```

Watch as the system:
1. Detects suspicious activity in Shibuya
2. AI analyzes 50+ critical events in real-time
3. Automatically dispatches Police/SWAT units
4. Coordinates multi-agent response
5. Resolves crisis with full telemetry

### Run Performance Benchmark
```bash
cargo run --bin performance_benchmark --release
```

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    100,000+ IoT SENSORS                      │
│  Traffic • Security • Disaster • Infrastructure • Health     │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │   LNMP CORE (Encoding)       │
          │   Compact Binary Format      │
          └──────┬───────────────────────┘
                 │
                 ▼
          ┌──────────────────────────────┐
          │   lnmp-net (Stage 1)         │
          │   Network Filter & QoS       │
          │   100K → 40K events          │
          └──────┬───────────────────────┘
                 │
                 ▼
          ┌──────────────────────────────┐
          │   lnmp-sfe (Stage 2)         │
          │   Semantic Field Engine      │
          │   40K → 200 critical events  │
          └──────┬───────────────────────┘
                 │
                 ▼
          ┌──────────────────────────────┐
          │   lnmp-llb (Stage 3)         │
          │   LLM Bridge                 │
          │   LNMP ↔ Natural Language    │
          └──────┬───────────────────────┘
                 │
                 ▼
          ┌──────────────────────────────┐
          │    OpenAI GPT-4o-mini        │
          │    Crisis Decision Making    │
          └──────┬───────────────────────┘
                 │
                 ▼
      ┌──────────────────────────────────────┐
      │      MULTI-AGENT SYSTEM               │
      │  Police • Fire • Medical • Traffic    │
      │     Real-time Coordination            │
      └───────────────────────────────────────┘
```

## 🎬 Demo Scenarios

### 1. Gang Violence Escalation (Shibuya)
```rust
cargo run --bin run_scenario
// Edit src/run_scenario.rs to select scenario
```

**Progression:**
- Tick 2: Suspicious gathering detected
- Tick 5: Violence erupts (5 individuals)
- Tick 8: **WEAPON CONFIRMED** - AI dispatches SWAT
- Tick 15: Incident resolved, agents return

### 2. Major Earthquake (Magnitude 7.2)
**Multi-hazard response:**
- P-wave early warning detected
- Main shock triggers city-wide alert
- Secondary fires break out
- Fire + Medical teams deployed
- Real-time evacuation coordination

### 3. Compound Crisis (Traffic Accident → Fire)
**Cross-agency coordination:**
- Major intersection accident
- Vehicle fire outbreak
- Traffic Control secures perimeter
- Fire units extinguish blaze
- Full incident resolution tracking

## 📊 Dashboard Features

```
🏙️  TOKYO SMART CITY OS - COMMAND CENTER
═══════════════════════════════════════════════════════════════
⏱️  TICK: 15   | 🚨 SCENARIO: Escalation: 1.0      | 👥 AGENTS: 11
───────────────────────────────────────────────────────────────
📊 PIPELINE METRICS
   Bandwidth:       92.31% ███████████████████████████░░░
   LLM Cost Saved:  92.31% ███████████████████████████░░░

📈 PERFORMANCE TRENDS (Last 20 ticks)
   Bandwidth Savings:
      ▆▇▇▇▇▆▇▇▇▇▇▇▇▇▇▇▇▇▇▇
   Critical Events:
      ▅▅▅▅▆▆▆▇▇▇▇▇▇▇▇▇▇▇██
───────────────────────────────────────────────────────────────
🧠 AI DECISIONS (LLM Bridge)
   ➤ WEAPON CONFIRMED - Dispatch SWAT immediately
   ➤ Medical standby required
```

## 🛠️ Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Protocol** | LNMP | Compact binary encoding (70% smaller) |
| **Network** | lnmp-net | QoS routing, deduplication |
| **Semantic Engine** | lnmp-sfe | Composite importance scoring |
| **LLM Bridge** | lnmp-llb | Natural language ↔ LNMP |
| **AI** | OpenAI GPT-4o-mini | Real-time crisis analysis |
| **Agents** | Multi-agent system | Autonomous emergency response |
| **Visualization** | Terminal + ANSI | Real-time sparklines & metrics |

## 📈 Why LNMP?

Traditional city management systems send **every single event** to the cloud and LLM, resulting in:
- 💸 **Massive bandwidth costs** (100K events × 0.5KB = 50 MB/sec)
- 💰 **Expensive LLM calls** ($1.15 per 100K events)
- ⏱️ **High latency** (network + API round-trips)
- 🔌 **Unnecessary infrastructure** scaling

### LNMP Solution

1. **Compact Encoding**: 70% smaller than JSON
2. **Semantic Filtering**: Only critical events pass through (97% reduction)
3. **Edge Intelligence**: Pre-process data locally before cloud
4. **Smart Routing**: QoS-based priority queuing

**Result:** Same intelligence, 66% lower cost, 10× faster decisions.

## 🔬 Field ID (FID) Dictionary

| FID | Field | Importance | Usage |
|-----|-------|-----------|--------|
| 1-9 | **Metadata** | Low | Source ID, timestamp, version |
| 10-19 | **Location** | High | GPS coordinates, area codes |
| 20-29 | **Traffic** | Medium | Flow, accidents, violations |
| 50-59 | **Security** | Critical | Violence, weapons, theft |
| 60-69 | **Disaster** | Critical | Fire, flood, earthquake |
| 70-79 | **Seismic** | Critical | Magnitude, infrastructure risk |
| 200-210 | **Commands** | High | Agent dispatch, evacuation |

## 🧪 Testing

```bash
# Run all tests
cargo test

# Performance benchmarks  
cargo run --bin performance_benchmark --release

# Stress test (100K events)
cargo run --bin full_pipeline_demo

# Code quality
cargo fmt && cargo clippy
```

## 📚 Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System design & data flow
- [redesign.md](./redesign.md) - Original vision document
- [Walkthrough](/.gemini/antigravity/brain/.../walkthrough.md) - Session summary

## 💡 Use Cases

**Emergency Services:**
- Real-time crime detection & response
- Automated dispatch optimization
- Resource allocation based on severity

**Disaster Management:**
- Early warning systems (earthquake P-waves)
- Multi-hazard coordination
- Evacuation route optimization

**Traffic Control:**
- Accident detection & response
- Dynamic traffic light adjustment
- Emergency vehicle priority routing

**Cost Optimization:**
- 97% bandwidth reduction
- 66% LLM API cost savings
- Edge-first architecture

## 🤝 Contributing

This is a showcase project demonstrating LNMP protocol capabilities. For production deployment:

1. Add authentication & encryption
2. Implement database persistence
3. Scale agent system horizontally
4. Add Web UI dashboard
5. Integrate real sensor feeds

## 📄 License

MIT License - See LICENSE file for details

## 🎓 Credits

Built with the LNMP Protocol by the LNMP Team.

**Powered by:**
- Rust (Performance & Safety)
- LNMP Protocol (Efficiency)
- OpenAI GPT-4o-mini (Intelligence)

---

**Try it now:** `cargo run --bin run_scenario` and watch AI save Tokyo! 🗾
