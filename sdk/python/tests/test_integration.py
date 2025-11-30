"""Integration tests for LNMP Python SDK.

Tests end-to-end workflows across all modules.
"""

import unittest
try:
    import lnmp
    LNMP_AVAILABLE = True
except ImportError:
    LNMP_AVAILABLE = False


@unittest.skipIf(not LNMP_AVAILABLE, "lnmp not installed")
class TestIntegration(unittest.TestCase):
    """End-to-end integration tests."""

    def test_parse_encode_roundtrip(self):
        """Test text parse and encode roundtrip."""
        original_text = "F12=14532;F7=1"
        
        # Parse
        record = lnmp.core.parse(original_text)
        
        # Encode back
        encoded = record.encode()
        
        # Should contain same fields (order may differ)
        self.assertIn("F12=14532", encoded)
        self.assertIn("F7=1", encoded)

    def test_binary_roundtrip(self):
        """Test binary encoding roundtrip."""
        text = "F12=14532;F7=1"
        
        # Parse from text
        record = lnmp.core.parse(text)
        
        # Encode to binary
        binary = record.encode_binary()
        self.assertIsInstance(binary, bytes)
        self.assertGreater(len(binary), 0)
        
        # Decode from binary
        decoded = lnmp.core.decode_binary(binary)
        
        # Encode to text
        decoded_text = decoded.encode()
        
        # Verify fields present
        self.assertIn("F12=14532", decoded_text)
        self.assertIn("F7=1", decoded_text)

    def test_full_envelope_workflow(self):
        """Test complete envelope workflow: parse → wrap → score → route."""
        # 1. Parse record
        record = lnmp.core.parse("F12=14532;F7=1")
        
        # 2. Wrap with envelope
        envelope = lnmp.envelope.wrap(
            record,
            source="test-service",
            trace_id="test-123"
        )
        
        # 3. Score context
        score = lnmp.net.context_score(envelope)
        self.assertIn("composite", score.__dict__)
        self.assertIsInstance(score.composite, float)
        
        # 4. Get routing decision
        decision = lnmp.net.routing_decide(envelope)
        self.assertIsInstance(decision, str)

    def test_embedding_delta_workflow(self):
        """Test embedding delta compression workflow."""
        base = [0.1, 0.2, 0.3, 0.4]
        updated = [0.1, 0.25, 0.3, 0.4]  # Changed one value
        
        # Compute delta
        change_count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        # Should have change information
        self.assertEqual(change_count, 1)
        self.assertIsInstance(delta_bytes, bytes)

    def test_spatial_roundtrip(self):
        """Test spatial encoding roundtrip."""
        x, y, z = 1.5, 2.5, 3.5
        
        # Encode
        encoded = lnmp.spatial.encode_position3d(x, y, z)
        self.assertIsInstance(encoded, bytes)
        
        # Decode
        decoded_x, decoded_y, decoded_z = lnmp.spatial.decode_position3d(encoded)
        
        # Verify (with float tolerance)
        self.assertAlmostEqual(decoded_x, x, places=2)
        self.assertAlmostEqual(decoded_y, y, places=2)
        self.assertAlmostEqual(decoded_z, z, places=2)

    def test_quantization_workflow(self):
        """Test vector quantization."""
        vector = [0.1, 0.2, 0.3, 0.4, 0.5]
        
        # Quantize to QInt8
        quantized = lnmp.utils.quantize(vector, "QInt8")
        self.assertIsInstance(quantized, bytes)
        
        # Should be smaller than original (float32 = 4 bytes each)
        original_size = len(vector) * 4
        self.assertLess(len(quantized), original_size)

    def test_sanitize_workflow(self):
        """Test text sanitization."""
        dirty = "F12= 14532 ; F7=1  "
        
        # Sanitize
        clean = lnmp.utils.sanitize(dirty)
        
        # Should be normalized
        self.assertIsInstance(clean, str)
        # No leading/trailing spaces after semicolon
        self.assertNotIn("  ", clean)

    def test_debug_explain_workflow(self):
        """Test debug explanation."""
        text = "F12=14532"
        
        # Get explanation
        explained = lnmp.utils.debug_explain(text)
        
        # Should contain field information
        self.assertIsInstance(explained, str)
        self.assertIn("F12", explained)

    def test_llm_normalize_and_route(self):
        """Test high-level LLM workflow."""
        result = lnmp.llm.normalize_and_route(
            "F12=14532;F7=1",
            source="test-service",
            threshold=0.7
        )
        
        # Should have all keys
        self.assertIn("record", result)
        self.assertIn("envelope", result)
        self.assertIn("score", result)
        self.assertIn("decision", result)
        self.assertIn("send_to_llm", result)
        
        # send_to_llm should be boolean
        self.assertIsInstance(result["send_to_llm"], bool)


if __name__ == "__main__":
    unittest.main()
