"""Unit tests for lnmp.utils module."""

import unittest
import lnmp

class TestUtils(unittest.TestCase):
    """Test utility functions."""
    
    def test_quantize_qint8(self):
        """Test QInt8 quantization."""
        vector = [0.1, 0.2, 0.3, 0.4, 0.5]
        
        quantized = lnmp.utils.quantize(vector, "QInt8")
        
        self.assertIsInstance(quantized, bytes)
        self.assertGreater(len(quantized), 0)
        # QInt8 should be smaller than float32
        self.assertLess(len(quantized), len(vector) * 4)
    
    def test_quantize_qint4(self):
        """Test QInt4 quantization."""
        vector = [0.1, 0.2, 0.3, 0.4, 0.5]
        
        quantized = lnmp.utils.quantize(vector, "QInt4")
        
        self.assertIsInstance(quantized, bytes)
        # QInt4 should be even smaller
        self.assertLess(len(quantized), len(vector) * 2)
    
    def test_quantize_binary(self):
        """Test binary quantization."""
        vector = [0.1, -0.2, 0.3, -0.4, 0.5]
        
        quantized = lnmp.utils.quantize(vector, "Binary")
        
        self.assertIsInstance(quantized, bytes)
        # Binary should be very compact
        self.assertLess(len(quantized), len(vector))
    
    def test_sanitize_basic(self):
        """Test basic sanitization."""
        dirty = "F12=14532 ; F7=1  "
        
        clean = lnmp.utils.sanitize(dirty)
        
        self.assertIsInstance(clean, str)
        self.assertIn("F12=14532", clean)
        self.assertIn("F7=1", clean)
    
    def test_sanitize_removes_extra_spaces(self):
        """Test that sanitization removes extra spaces."""
        dirty = "F12=  14532  ;  F7=  1  "
        
        clean = lnmp.utils.sanitize(dirty)
        
        # Should not have multiple consecutive spaces
        self.assertNotIn("  ", clean)
    
    def test_sanitize_preserves_quoted_spaces(self):
        """Test that sanitization preserves spaces in quoted strings."""
        dirty = "F3=\"hello  world\""
        
        clean = lnmp.utils.sanitize(dirty)
        
        # Should preserve spaces inside quotes
        self.assertIn("hello  world", clean)
    
    def test_debug_explain(self):
        """Test debug explanation."""
        text = "F12=14532;F7=1"
        
        explanation = lnmp.utils.debug_explain(text)
        
        self.assertIsInstance(explanation, str)
        self.assertGreater(len(explanation), 0)
        # Should contain field information
        self.assertIn("F12", explanation)
    
    def test_debug_explain_detailed(self):
        """Test that debug explanation is detailed."""
        text = "F12=14532"
        
        explanation = lnmp.utils.debug_explain(text)
        
        # Should be informative
        self.assertGreater(len(explanation), 5)

if __name__ == "__main__":
    unittest.main()
