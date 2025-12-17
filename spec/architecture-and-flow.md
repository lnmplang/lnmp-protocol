# LNMP Protocol Architecture (v0.5.14)

This document defines the structural architecture of the LNMP protocol, mapping the **12 ecosystem crates** into a unified layered model.

## 1. The Protocol Stack (The Centipede)

The LNMP architecture is composed of 5 vertical layers. Each layer has a specific responsibility and communicates only with adjacent layers.

```mermaid
graph TD
    %% --- LAYER 1: APPLICATION & INTELLIGENCE ---
    subgraph L1 ["1. Application Layer (Intelligence)"]
        direction TB
        App[User Application]
        SFE[lnmp-sfe: Semantic Field Engine]
        LLB[lnmp-llb: LLM Bridge]
        
        App -->|"Intent"| SFE
        SFE -->|"FIDs"| LLB
        LLB -->|"Context"| LLM((AI Model))
    end

    %% --- LAYER 2: CORE & PRESENTATION ---
    subgraph L2 ["2. Core Layer (Definition)"]
        direction TB
        Core[lnmp-core: Registry & Types]
        Codec[lnmp-codec: Serialization]
        San[lnmp-sanitize: Input Filter]
    end

    %% --- LAYER 3: CAPABILITIES (EXTENSIONS) ---
    subgraph L3 ["3. Capability Layer (Extensions)"]
        direction LR
        Spatial[lnmp-spatial: 3D/Geo]
        Emb[lnmp-embedding: Vectors]
        Quant[lnmp-quant: Compression]
        
        Spatial -.-> Core
        Emb -.-> Core
        Quant -.-> Emb
    end
    
    %% --- LAYER 4: DEFENSE & IDENTITY ---
    subgraph L4 ["4. Session Layer (Defense)"]
        direction TB
        Env[lnmp-envelope: Identity/Trace]
    end

    %% --- LAYER 5: TRANSPORT & ROUTING ---
    subgraph L5 ["5. Network Layer (Transport)"]
        direction TB
        Net[lnmp-net: QoS/Routing]
        Trans[lnmp-transport: Bindings]
        Ext[External Network]
    end

    %% --- VERTICAL FLOW (THE SPINE) ---
    SFE ==> Core
    Core ==> Codec
    Codec ==> Env
    Env ==> Net
    Net ==> Trans
    Trans ==> Ext
    
    %% --- INPUT FLOW ---
    Ext -.-> San
    San -.-> Core
    
    %% STYLING
    classDef layer fill:#f9f9f9,stroke:#333,stroke-width:2px;
    class L1,L2,L3,L4,L5 layer;
```

### Module Responsibilities

| Layer | Module | Responsibility |
| :--- | :--- | :--- |
| **1. App** | `lnmp-sfe` | **Translator:** Converts human intent/concepts into Protocol FIDs. |
| | `lnmp-llb` | **Optimizer:** Formats data for LLM context windows (ShortForm). |
| **2. Core** | `lnmp-core` | **Authority:** Holds the FID Registry and Type Definitions. |
| | `lnmp-codec` | **Encoder:** Handles Binary/Text serialization. |
| | `lnmp-sanitize` | **Filter:** Cleans "dirty" input before strict parsing. |
| **3. Cap** | `lnmp-spatial` | **Sense:** Provides coordinate systems and spatial math. |
| | `lnmp-embedding` | **Memory:** Manages high-dimensional vector data. |
| | `lnmp-quant` | **Efficiency:** Compresses vectors (up to 32x) for transport. |
| **4. Def** | `lnmp-envelope` | **Identity:** Signs packets, adds Trace IDs and Timestamps. |
| **5. Net** | `lnmp-net` | **Router:** Classifies messages (Alert/Log) and assigns QoS. |
| | `lnmp-transport` | **Binding:** Maps LNMP concepts to HTTP/Kafka/gRPC. |

---

## 2. The Data Flow (The DNA Helix)

This sequence illustrates the lifecycle of a message, showing how two agents sync ("Bonding") and exchange data ("Synthesis").

```mermaid
sequenceDiagram
    participant A as 👤 Agent A (Sender)
    participant B as 🤖 Agent B (Receiver)
    
    Note over A,B: PHASE 1: DNA BONDING (Sync)
    
    A->>B: 🟡 Hello (Protocol v0.5)
    B-->>A: 🟢 Welcome (Registry v1.2)
    
    rect rgb(240, 240, 240)
    Note left of A: Compatibility Check
    A->>A: Local v1.0 vs Remote v1.2
    alt Registry Mismatch
        A->>B: 🔴 RequestDelta
        B-->>A: 🔵 RegistryDelta (New FIDs)
        A->>A: Apply & Recompile
    end
    end
    
    A->>B: 🟡 Ready (Bonded)
    
    Note over A,B: PHASE 2: SYNTHESIS (Exchange)
    
    loop Real-time Interaction
        A->>B: 📦 [Env: TraceID] [Msg: Intent] (FIDs: 10, 100)
        
        rect rgb(230, 240, 255)
            Note right of B: Internal Processing
            B->>B: Net (QoS) -> Env (Verify) -> Codec (Decode)
            B->>B: Core (Validate) -> SFE (Understand)
        end
        
        B-->>A: 📦 [Msg: Response] [Meta: Confidence=0.99]
        
        opt Spatial Reflex
            A->>B: 🧊 [Spatial: Position Update] (F256)
            B-->>A: 🧊 [Spatial: New Coordinates]
        end
    end
    
    Note over A,B: PHASE 3: TERMINATION
    
    A->>B: 🔴 Close Session
    B-->>A: 🔴 Ack
```
