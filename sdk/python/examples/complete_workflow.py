"""
Complete LLM Workflow Example

Demonstrates the full LNMP workflow for LLM context optimization.
"""

import lnmp

def main():
    print("=== LNMP Complete Workflow ===\n")
    
    # Simulated incoming data
    raw_data = "F12=14532;F7=1;F23=[admin,developer];F50=high_priority"
    
    print(f"Raw data: {raw_data}\n")
    
    # Use the high-level workflow helper
    result = lnmp.llm.normalize_and_route(
        raw_data,
        source="api-gateway",
        trace_id="workflow-demo-001",
        threshold=0.7
    )
    
    print("=== Workflow Results ===")
    print(f"✓ Record parsed")
    print(f"✓ Envelope wrapped")
    print(f"✓ Context scored: {result['score'].composite:.3f}")
    print(f"✓ Routing decision: {result['decision']}")
    print(f"✓ Send to LLM: {result['send_to_llm']}")
    
    # Make routing decision
    if result['send_to_llm']:
        print("\n→ Routing to LLM for processing...")
        # Here you would send to your LLM
        print("  (LLM processing would happen here)")
    else:
        print("\n→ Processing locally...")
        # Handle locally
        print("  (Local processing would happen here)")
    
    # Optional: Advanced features
    print("\n=== Advanced Features ===")
    
    # Spatial data
    print("\n1. Spatial encoding:")
    x, y, z = 10.5, 20.3, 5.0
    spatial_bytes = lnmp.spatial.encode_position3d(x, y, z)
    print(f"   Position ({x}, {y}, {z}) → {len(spatial_bytes)} bytes")
    
    # Quantization
    print("\n2. Vector quantization:")
    vector = [0.1, 0.2, 0.3, 0.4, 0.5]
    quantized = lnmp.utils.quantize(vector, "QInt8")
    print(f"   Vector {len(vector)} dims → {len(quantized)} bytes compressed")
    
    # Sanitization
    print("\n3. Input sanitization:")
    dirty = "F12= 14532 ; F7=1  "
    clean = lnmp.utils.sanitize(dirty)
    print(f"   Dirty:  '{dirty}'")
    print(f"   Clean:  '{clean}'")
    
    print("\n✅ Complete workflow finished!")

if __name__ == "__main__":
    main()
