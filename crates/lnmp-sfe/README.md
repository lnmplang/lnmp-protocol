# lnmp-sfe

Semantic Fidelity Engine for LNMP (LLM Native Minimal Protocol).

> **FID Registry:** All examples use official Field IDs from [`registry/fids.yaml`](../../registry/fids.yaml).

This crate provides the semantic dictionary system that maps field IDs to human-readable names and provides equivalence mappings for semantic normalization.

## Features

- **Semantic Dictionary**: Maps field IDs to human-readable names
- **Equivalence Mappings**: Define synonym relationships for semantic normalization
- **YAML Support**: Load dictionaries from YAML files

## Usage

```rust
use lnmp_sfe::SemanticDictionary;

// Load dictionary from YAML file
let dict = SemanticDictionary::load_from_file("dictionary.yaml")?;

// Get field name
if let Some(name) = dict.get_field_name(12) {
    println!("Field 12 is: {}", name);
}

// Get equivalence mapping
if let Some(canonical) = dict.get_equivalence(7, "yes") {
    println!("'yes' maps to: {}", canonical);
}
```

## Dictionary Format

```yaml
fields:
  12:
    name: user_id
    type: integer
  7:
    name: is_active
    type: boolean
    equivalences:
      yes: "1"
      true: "1"
      no: "0"
      false: "0"
  23:
    name: roles
    type: string_array
    equivalences:
      admin: administrator
      dev: developer
```

## Context Profiling

The Semantic Fidelity Engine includes **context profiling** capabilities to help LLMs prioritize which records to use in RAG (Retrieval-Augmented Generation) systems and other applications.

### Quick Start

```rust
use lnmp_sfe::{ContextScorer, ContextPrioritizer, ScoringWeights};
use lnmp_envelope::EnvelopeBuilder;

// Create scorer
let scorer = ContextScorer::new();

// Score an envelope
let now = current_timestamp_ms();
let profile = scorer.score_envelope(&envelope, now);

println!("Freshness: {:.2}", profile.freshness_score);
println!("Importance: {}", profile.importance);
println!("Risk: {:?}", profile.risk_level);
println!("Confidence: {:.2}", profile.confidence);
```

### Scoring Components

- **Freshness (0.0-1.0)**: Exponential decay based on timestamp age (default: 24h decay)
- **Importance (0-255)**: Field-level priority from dictionary or default
- **Risk Level**: Low/Medium/High/Critical based on source trustworthiness
- **Confidence (0.0-1.0)**: Data reliability (+0.2 boost for trusted sources)

### RAG Use Cases

```rust
// Select top-K contexts for LLM prompt
let weights = ScoringWeights::new(0.8, 0.1, 0.1); // 80% freshness
let top_5 = ContextPrioritizer::select_top_k(contexts, 5, weights);

// Filter by criteria
let high_quality = ContextPrioritizer::filter_by_threshold(contexts, 0.6);
let very_fresh = ContextPrioritizer::filter_by_freshness(contexts, 0.9);

// Rank all contexts
let ranked = ContextPrioritizer::rank_for_llm(contexts, weights);
```

### Dictionary Integration

Add importance levels to YAML:

```yaml
fields:
  12:
    name: user_id
    type: integer
    importance: 200  # High importance (0-255)
```

## Examples

- `context_scoring.rs` - Basic context scoring
- `rag_prioritization.rs` - RAG system prioritization

```bash
cargo run --example context_scoring
cargo run --example rag_prioritization
```

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
