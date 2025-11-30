"""Tests for Transport and Embedding Delta features."""

import unittest
import lnmp
from lnmp.envelope import wrap
from lnmp.core import parse

class TestTransport(unittest.TestCase):
    def test_http_headers_roundtrip(self):
        # Create an envelope
        record = parse("F1=1")
        envelope = wrap(record, source="test-service", trace_id="trace-123")
        
        # To headers
        headers = lnmp.transport.to_http_headers(envelope)
        self.assertEqual(headers.get("x-lnmp-source"), "test-service")
        self.assertEqual(headers.get("x-lnmp-trace-id"), "trace-123")
        
        # From headers
        new_envelope = lnmp.transport.from_http_headers(headers)
        self.assertEqual(new_envelope.source, "test-service")
        self.assertEqual(new_envelope.trace_id, "trace-123")

class TestEmbeddingDelta(unittest.TestCase):
    def test_delta_apply(self):
        base = [1.0, 2.0, 3.0, 4.0]
        updated = [1.0, 2.5, 3.0, 4.5] # Changed indices 1 and 3
        
        # Compute delta
        change_count, delta_bytes = lnmp.embedding.delta(base, updated)
        self.assertEqual(change_count, 2)
        self.assertIsInstance(delta_bytes, bytes)
        self.assertGreater(len(delta_bytes), 0)
        
        # Apply delta
        restored = lnmp.embedding.apply_delta(base, delta_bytes)
        
        # Verify
        self.assertEqual(len(restored), 4)
        self.assertAlmostEqual(restored[1], 2.5)
        self.assertAlmostEqual(restored[3], 4.5)
        self.assertEqual(restored[0], 1.0)

if __name__ == "__main__":
    unittest.main()
