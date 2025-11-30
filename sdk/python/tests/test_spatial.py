"""Unit tests for lnmp.spatial module."""

import unittest
import lnmp

class TestSpatial(unittest.TestCase):
    """Test spatial encoding/decoding."""
    
    def test_encode_position3d(self):
        """Test encoding 3D position."""
        encoded = lnmp.spatial.encode_position3d(1.0, 2.0, 3.0)
        
        self.assertIsInstance(encoded, bytes)
        self.assertGreater(len(encoded), 0)
    
    def test_decode_position3d(self):
        """Test decoding 3D position."""
        x, y, z = 123.456, 789.012, 345.678
        
        encoded = lnmp.spatial.encode_position3d(x, y, z)
        decoded_x, decoded_y, decoded_z = lnmp.spatial.decode_position3d(encoded)
        
        # Should match with floating point tolerance
        self.assertAlmostEqual(decoded_x, x, places=2)
        self.assertAlmostEqual(decoded_y, y, places=2)
        self.assertAlmostEqual(decoded_z, z, places=2)
    
    def test_roundtrip_zero_position(self):
        """Test roundtrip with zero position."""
        x, y, z = 0.0, 0.0, 0.0
        
        encoded = lnmp.spatial.encode_position3d(x, y, z)
        decoded_x, decoded_y, decoded_z = lnmp.spatial.decode_position3d(encoded)
        
        self.assertAlmostEqual(decoded_x, x, places=2)
        self.assertAlmostEqual(decoded_y, y, places=2)
        self.assertAlmostEqual(decoded_z, z, places=2)
    
    def test_roundtrip_negative_values(self):
        """Test roundtrip with negative values."""
        x, y, z = -10.5, -20.3, -30.7
        
        encoded = lnmp.spatial.encode_position3d(x, y, z)
        decoded_x, decoded_y, decoded_z = lnmp.spatial.decode_position3d(encoded)
        
        self.assertAlmostEqual(decoded_x, x, places=2)
        self.assertAlmostEqual(decoded_y, y, places=2)
        self.assertAlmostEqual(decoded_z, z, places=2)
    
    def test_roundtrip_large_values(self):
        """Test roundtrip with large values."""
        x, y, z = 10000.5, 20000.3, 30000.7
        
        encoded = lnmp.spatial.encode_position3d(x, y, z)
        decoded_x, decoded_y, decoded_z = lnmp.spatial.decode_position3d(encoded)
        
        self.assertAlmostEqual(decoded_x, x, places=1)
        self.assertAlmostEqual(decoded_y, y, places=1)
        self.assertAlmostEqual(decoded_z, z, places=1)
    
    def test_encoding_stability(self):
        """Test that encoding is deterministic."""
        x, y, z = 1.5, 2.5, 3.5
        
        encoded1 = lnmp.spatial.encode_position3d(x, y, z)
        encoded2 = lnmp.spatial.encode_position3d(x, y, z)
        
        self.assertEqual(encoded1, encoded2)
    
    def test_encoded_size(self):
        """Test that encoded size is compact."""
        encoded = lnmp.spatial.encode_position3d(1.0, 2.0, 3.0)
        
        # Should be compact (around 13 bytes based on benchmarks)
        self.assertLessEqual(len(encoded), 20)

if __name__ == "__main__":
    unittest.main()
