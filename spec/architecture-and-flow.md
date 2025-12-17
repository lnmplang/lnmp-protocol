# LNMP Architecture & Flow: The Digital Organism

This document provides a comprehensive, end-to-end visualization of the LNMP protocol (v0.5.14). It maps the **complete 12-crate ecosystem** to biological systems, illustrating how the protocol functions as a living, breathing language for AI agents.

## 1. The Anatomy of LNMP (Module Map)

Every single module plays a vital role in the physiology of the digital organism.

```mermaid
graph TD
    %% ZONE 1: THE MIND (Cognitive Interface)
    subgraph "The Cortex (Cognition & Interface)"
        App[Application Logic]
        SFE[lnmp-sfe: Semantic Field Engine]
        LLB[lnmp-llb: LLM Bridge / Language Center]
        
        App -->|Intent| SFE
        SFE -->|Semantic FIDs| LLB
        LLB -->|Optimized Tokens| LLM[LLM Model]
    end

    %% ZONE 2: THE CORE (Genetic Material)
    subgraph "The Nucleus (DNA & Structure)"
        Core[lnmp-core: Types & Registry]
        Codec[lnmp-codec: Encoding & Rules]
    end

    %% ZONE 3: PROTECTION (Immune System)
    subgraph "Immune System (Protection & Hygiene)"
        Env[lnmp-envelope: Cell Wall / Identity]
        San[lnmp-sanitize: Filter / Input Hygiene]
    end

    %% ZONE 4: PHYSIOLOGY (Capabilities)
    subgraph "Physiology (Senses & Metabolism)"
        Spatial[lnmp-spatial: Proprioception / Space]
        Emb[lnmp-embedding: Vector Memory]
        Quant[lnmp-quant: Metabolism / Compression]
    end

    %% ZONE 5: CIRCULATION (Network & Flow)
    subgraph "Nervous & Circulatory System"
        Net[lnmp-net: Autonomic System / Routing]
        Trans[lnmp-transport: Vascular System / Bindings]
        Ext[External Network]
    end

    %% Interconnections
    SFE -->|Maps Meaning| Core
    Core -->|Defines| Codec
    
    %% Input Flow
    App --> San
    San -->|Clean Input| Codec
    
    %% Capability extensions
    Spatial -->|Extends| Core
    Emb -->|Extends| Core
    Quant -->|Optimizes| Emb

    %% Output Flow
    Codec -->|Payload| Env
    Env -->|Packet| Net
    Net -->|QoS & Priority| Trans
    Trans -->|HTTP/gRPC/Kafka| Ext
```

### 🫀 Organ Systems Defined

1.  **The Cortex (Cognition):**
    *   **`lnmp-sfe` (Semantic Field Engine):** "Broca's Area". Translates abstract thought (Intent) into protocol language (FIDs).
    *   **`lnmp-llb` (LLM Bridge):** "Language Center". Optimizes data for the LLM brain (ShortForm/Explain Mode), saving cognitive energy (tokens).

2.  **The Nucleus (Genetics):**
    *   **`lnmp-core`:** "DNA". The immutable Registry and Base Types.
    *   **`lnmp-codec`:** "RNA Polymerase". Transcribes DNA into transmissible binary strands.

3.  **Physiology (Capabilities):**
    *   **`lnmp-spatial`:** "Proprioception". Allows the agent to sense itself in 3D/Geo space.
    *   **`lnmp-embedding`:** "Hippocampus". Long-term vector memory structure.
    *   **`lnmp-quant`:** "Metabolism". Compresses heavy vector data (32x) for efficiency.

4.  **Immune System (Defense):**
    *   **`lnmp-envelope`:** "Cell Wall". Defines Identity, Traceability, and protects the payload.
    *   **`lnmp-sanitize`:** "Liver/Kidneys". Filters and fixes toxic (malformed) input before it reaches the core.

5.  **Circulation (Transport):**
    *   **`lnmp-net`:** "Autonomic Nervous System". Decides *how* to route signals (QoS, Priority) based on meaning.
    *   **`lnmp-transport`:** "Vascular System". The physical bindings (HTTP headers, Kafka metadata) that carry the blood.

---

## 2. The Cycle of Life: E2E Data Flow

The complete journey of a message within the organism.

```mermaid
sequenceDiagram
    participant LLM as � LLM/App
    participant LLB as �️ LLB (Bridge)
    participant San as � Sanitize
    participant SFE as � SFE (Intent)
    participant Core as 🧬 Core/Codec
    participant Net as ⚡ Net/Trans

    Note over LLM, Net: 1. OUTBOUND (Expression)
    LLM->>SFE: Intent: "High Priority Alert"
    SFE->>Core: Map to FIDs (Alert=1024, TTL=500ms)
    Core->>Core: Validation
    Core->>Net: Binary Payload
    Net->>Net: Assign QoS=255 (Critical)
    Net->>Net: Route -> "Direct Peer" (Bypass Cloud)
    Net->>Ext: Dispatch via UDP/QUIC

    Note over LLM, Net: 2. INBOUND (Perception)
    Ext->>San: Raw Input Stream
    San->>San: Clean/Normalize
    San->>Core: Strict Parse
    Core->>Env: Verify Identity/Sig
    Core->>LLB: Decode for Consumption
    LLB->>LLM: ShortForm: "1024=Alert 768=Fire"
```

## 3. Why Every Module Matters

*   **Without `lnmp-sanitize`**, the organism is vulnerable to "poison" (malformed data) crashing the Core.
*   **Without `lnmp-llb`**, the "Brain" (LLM) overflows with verbose data (Context Window Exhaustion).
*   **Without `lnmp-net`**, the system has "high blood pressure" (network congestion) because it treats all data as equal, blocking critical reflexes with mundane logs.
*   **Without `lnmp-quant`**, the "Memory" (Vectors) becomes too heavy to move, paralyzing the agent.

This architecture ensures **Safety, Efficiency, and Intelligence** at every layer.
