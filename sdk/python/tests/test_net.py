"""Unit tests for lnmp.net module."""

import unittest
import lnmp

class TestNet(unittest.TestCase):
    """Test network routing and scoring functionality."""
    
    def test_context_score(self):
        """Test context scoring."""
        record = lnmp.core.parse("F12=14532;F7=1")
        envelope = lnmp.envelope.wrap(record, source="test-service")
        
        score = lnmp.net.context_score(envelope)
        
        # Should have all score components
        self.assertHasAttr(score, 'composite')
        self.assertHasAttr(score, 'freshness')
        self.assertHasAttr(score, 'importance')
        
        # Scores should be floats
        self.assertIsInstance(score.composite, float)
        self.assertIsInstance(score.freshness, float)
        self.assertIsInstance(score.importance, float)
    
    def test_context_score_range(self):
        """Test that scores are in valid range."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(record, source="test")
        
        score = lnmp.net.context_score(envelope)
        
        # Scores should be between 0 and 1 (or reasonable range)
        self.assertGreaterEqual(score.composite, 0.0)
        self.assertLessEqual(score.composite, 1.0)
    
    def test_routing_decide(self):
        """Test routing decision."""
        record = lnmp.core.parse("F12=14532;F7=1")
        envelope = lnmp.envelope.wrap(record, source="test-service")
        
        decision = lnmp.net.routing_decide(envelope)
        
        self.assertIsInstance(decision, str)
        self.assertGreater(len(decision), 0)
    
    def test_should_send_to_llm_high_threshold(self):
        """Test LLM routing with high threshold."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(record, source="test")
        
        # Test with very high threshold (should likely be False)
        result = lnmp.net.should_send_to_llm(envelope, threshold=0.99)
        self.assertIsInstance(result, bool)
    
    def test_should_send_to_llm_low_threshold(self):
        """Test LLM routing with low threshold."""
        record = lnmp.core.parse("F12=14532")
        envelope = lnmp.envelope.wrap(record, source="test")
        
        # Test with very low threshold (should likely be True)
        result = lnmp.net.should_send_to_llm(envelope, threshold=0.01)
        self.assertIsInstance(result, bool)
    
    def assertHasAttr(self, obj, attr):
        """Helper to assert object has attribute."""
        self.assertTrue(hasattr(obj, attr), f"Object missing attribute: {attr}")

if __name__ == "__main__":
    unittest.main()
