"""Unit tests for lnmp.envelope module."""

import unittest
import lnmp
from datetime import datetime

class TestEnvelope(unittest.TestCase):
    """Test envelope wrapping functionality."""
    
    def test_wrap_basic(self):
        """Test basic envelope wrapping."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(record, source="test-service")
        
        self.assertIsNotNone(envelope)
        self.assertEqual(envelope.source, "test-service")
    
    def test_wrap_with_trace_id(self):
        """Test wrapping with trace ID."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(
            record,
            source="test-service",
            trace_id="trace-abc-123"
        )
        
        self.assertEqual(envelope.trace_id, "trace-abc-123")
    
    def test_wrap_with_timestamp(self):
        """Test wrapping with explicit timestamp."""
        record = lnmp.core.parse("F12=14532")
        timestamp_ms = 1732373147000
        
        envelope = lnmp.envelope.wrap(
            record,
            source="test-service",
            timestamp_ms=timestamp_ms
        )
        
        self.assertEqual(envelope.timestamp, timestamp_ms)
    
    def test_wrap_auto_timestamp(self):
        """Test automatic timestamp generation."""
        record = lnmp.core.parse("F12=14532")
        before = int(datetime.now().timestamp() * 1000)
        
        envelope = lnmp.envelope.wrap(record, source="test-service")
        
        after = int(datetime.now().timestamp() * 1000)
        
        self.assertIsNotNone(envelope.timestamp)
        self.assertGreaterEqual(envelope.timestamp, before - 1000)  # 1 sec tolerance
        self.assertLessEqual(envelope.timestamp, after + 1000)
    
    def test_envelope_properties(self):
        """Test envelope property access."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(
            record,
            source="my-service",
            trace_id="trace-xyz"
        )
        
        # All properties should be accessible
        self.assertIsInstance(envelope.source, str)
        self.assertIsInstance(envelope.trace_id, str)
        self.assertIsInstance(envelope.timestamp, int)

if __name__ == "__main__":
    unittest.main()
