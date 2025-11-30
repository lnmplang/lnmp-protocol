"""Unit tests for lnmp.embedding module."""

import unittest
import lnmp

class TestEmbedding(unittest.TestCase):
    """Test embedding operations."""
    
    def test_delta_basic(self):
        """Test basic delta computation."""
        base = [0.1, 0.2, 0.3, 0.4]
        updated = [0.1, 0.25, 0.3, 0.4]  # Changed one value
        
        count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        self.assertEqual(count, 1)
        self.assertIsInstance(delta_bytes, bytes)
        self.assertGreater(len(delta_bytes), 0)
    
    def test_delta_no_changes(self):
        """Test delta with no changes."""
        base = [0.1, 0.2, 0.3]
        updated = [0.1, 0.2, 0.3]
        
        count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        self.assertEqual(count, 0)
        self.assertIsInstance(delta_bytes, bytes)
    
    def test_delta_all_changed(self):
        """Test delta with all values changed."""
        base = [0.1, 0.2, 0.3, 0.4]
        updated = [0.2, 0.3, 0.4, 0.5]
        
        count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        self.assertEqual(count, 4)
        self.assertIsInstance(delta_bytes, bytes)
    
    def test_apply_delta(self):
        """Test applying delta to base vector."""
        base = [1.0, 2.0, 3.0, 4.0]
        updated = [1.0, 2.5, 3.0, 4.5]
        
        # Compute delta
        count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        # Apply delta
        restored = lnmp.embedding.apply_delta(base, delta_bytes)
        
        # Should match updated vector
        self.assertEqual(len(restored), len(updated))
        for i, (r, u) in enumerate(zip(restored, updated)):
            self.assertAlmostEqual(r, u, places=5, msg=f"Mismatch at index {i}")
    
    def test_apply_delta_large_vector(self):
        """Test delta with larger vectors."""
        base = [0.1 * i for i in range(128)]
        updated = [0.1 * i + (0.05 if i % 10 == 0 else 0) for i in range(128)]
        
        count, delta_bytes = lnmp.embedding.delta(base, updated)
        
        # Should have 13 changes (indices 0, 10, 20, ..., 120)
        self.assertEqual(count, 13)
        
        # Apply and verify
        restored = lnmp.embedding.apply_delta(base, delta_bytes)
        self.assertEqual(len(restored), 128)
        
        for i in range(128):
            self.assertAlmostEqual(restored[i], updated[i], places=5)
    
    def test_delta_dimension_mismatch(self):
        """Test delta with mismatched dimensions."""
        base = [0.1, 0.2, 0.3]
        updated = [0.1, 0.2, 0.3, 0.4]  # Different length
        
        with self.assertRaises(Exception):
            lnmp.embedding.delta(base, updated)

if __name__ == "__main__":
    unittest.main()
