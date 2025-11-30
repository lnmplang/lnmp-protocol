"""
Basic LNMP Python SDK Usage Example

Demonstrates core functionality: parsing, encoding, and envelopes.
"""

import lnmp

def main():
    print("=== LNMP Python SDK - Basic Usage ===\n")
    
    # 1. Parse LNMP text
    print("1. Parsing LNMP text:")
    text = "F12=14532;F7=1;F23=[admin,developer]"
    record = lnmp.core.parse(text)
    print(f"   Input:  {text}")
    print(f"   Parsed: {record}")
    
    # 2. Encode back to text
    print("\n2. Encoding to text:")
    encoded = record.encode()
    print(f"   Output: {encoded}")
    
    # 3. Binary encoding
    print("\n3. Binary encoding:")
    binary = record.encode_binary()
    print(f"   Binary size: {len(binary)} bytes")
    
    # 4. Decode from binary
    decoded = lnmp.core.decode_binary(binary)
    print(f"   Decoded: {decoded.encode()}")
    
    # 5. Wrap with envelope
    print("\n4. Envelope wrapping:")
    envelope = lnmp.envelope.wrap(
        record,
        source="example-service",
        trace_id="example-trace-123"
    )
    print(f"   Envelope created with metadata")
    
    # 6. Context scoring
    print("\n5. Context scoring:")
    score = lnmp.net.context_score(envelope)
    print(f"   Composite score: {score.composite:.3f}")
    print(f"   Freshness: {score.freshness:.3f}")
    print(f"   Importance: {score.importance:.3f}")
    
    # 7. Routing decision
    print("\n6. Routing decision:")
    decision = lnmp.net.routing_decide(envelope)
    print(f"   Decision: {decision}")
    
    print("\n✅ Basic usage complete!")

if __name__ == "__main__":
    main()
