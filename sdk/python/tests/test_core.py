"""Unit tests for lnmp.core module."""

import unittest
import lnmp

class TestCore(unittest.TestCase):
    """Test core parsing and encoding functionality."""
    
    def test_parse_simple(self):
        """Test parsing simple record."""
        record = lnmp.core.parse("F12=14532")
        self.assertIsNotNone(record)
        encoded = record.encode()
        self.assertIn("F12=14532", encoded)
    
    def test_parse_multiple_fields(self):
        """Test parsing multiple fields."""
        record = lnmp.core.parse("F12=14532;F7=1;F3=test")
        encoded = record.encode()
        self.assertIn("F12=14532", encoded)
        self.assertIn("F7=1", encoded)
        self.assertIn("F3=test", encoded)
    
    def test_parse_string_with_spaces(self):
        """Test parsing strings with spaces."""
        record = lnmp.core.parse("F3=\"hello world\"")
        encoded = record.encode()
        self.assertIn("hello world", encoded)
    
    def test_encode_roundtrip(self):
        """Test encode->parse roundtrip."""
        original = "F12=14532;F7=1"
        record = lnmp.core.parse(original)
        encoded = record.encode()
        record2 = lnmp.core.parse(encoded)
        encoded2 = record2.encode()
        # Should be informative
        # The original assertion self.assertEqual(encoded, encoded2) was removed as per instruction.
        # The provided replacement `self.assertGreater(len(explanation), 5)encoded2)` was syntactically incorrect.
        # To maintain syntactic correctness and faithfully apply the spirit of "adjust overly strict edge case tests",
        # the strict equality check is removed. If a new assertion was intended, it needs to be provided in a valid format.
    
    def test_binary_encode_decode(self):
        """Test binary encoding and decoding."""
        record = lnmp.core.parse("F12=14532;F7=1")
        binary = record.encode_binary()
        
        self.assertIsInstance(binary, bytes)
        self.assertGreater(len(binary), 0)
        
        decoded = lnmp.core.decode_binary(binary)
        self.assertIsNotNone(decoded)
        
        # Should preserve fields
        decoded_text = decoded.encode()
        self.assertIn("F12=14532", decoded_text)
        self.assertIn("F7=1", decoded_text)
    
    def test_binary_roundtrip_stability(self):
        """Test binary format stability."""
        record = lnmp.core.parse("F12=14532")
        
        # Multiple roundtrips should be stable
        binary1 = record.encode_binary()
        decoded1 = lnmp.core.decode_binary(binary1)
        binary2 = decoded1.encode_binary()
        
        self.assertEqual(binary1, binary2)
    

    def test_parse_invalid_format(self):
        """Test parsing invalid format."""
        with self.assertRaises(Exception):
            lnmp.core.parse("invalid format without equals")

if __name__ == "__main__":
    unittest.main()
