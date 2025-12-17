#!/usr/bin/env python3
"""
Validate .lnmp fixture files against the FID Registry.

This script parses all .lnmp files and checks that:
1. All FIDs used are defined in the registry
2. Value types match expected types (where detectable)

Usage:
    python scripts/validate-fixtures.py
"""

import re
import sys
import yaml
from pathlib import Path
from typing import Dict, List, Tuple, Optional

# Load registry
REGISTRY_PATH = Path(__file__).parent.parent / "registry" / "fids.yaml"

def load_registry() -> Dict[int, dict]:
    """Load FID registry from YAML."""
    with open(REGISTRY_PATH) as f:
        data = yaml.safe_load(f)
    
    registry = {}
    
    # Load core FIDs
    for entry in data.get("core", []):
        registry[entry["fid"]] = entry
    
    # Load standard FIDs
    for entry in data.get("standard", []):
        registry[entry["fid"]] = entry
    
    # Load extended FIDs
    for entry in data.get("extended", []):
        registry[entry["fid"]] = entry
    
    return registry

def extract_fids_from_lnmp(content: str) -> List[Tuple[int, str]]:
    """Extract FIDs and their values from LNMP text content."""
    fids = []
    
    for line in content.split('\n'):
        line = line.strip()
        
        # Skip empty lines and comments
        if not line or line.startswith('#'):
            continue
        
        # Pattern to match F<id>=<value> or F<id>:<hint>=<value>
        # Value ends at ; or \n or } or # (checksum)
        pattern = r'F(\d+)(?::[a-z]+)?=([^;\n\}#]+)'
        
        for match in re.finditer(pattern, line):
            fid = int(match.group(1))
            value = match.group(2).strip()
            fids.append((fid, value))
    
    return fids

def infer_type(value: str) -> Optional[str]:
    """Infer the type of a value string."""
    # Remove quotes if present
    value = value.strip()
    
    # Check for arrays
    if value.startswith('['):
        # Check if it's a nested array (record array)
        if value.startswith('[{'):
            return "RecordArray"
        # Check if it looks like numbers
        inner = value[1:-1].strip()
        if not inner:
            return "StringArray"  # Empty array
        # Try to detect type from first element
        first = inner.split(',')[0].strip()
        first = first.strip('"\'')
        if first in ('0', '1', 'true', 'false', 'True', 'False'):
            return "BoolArray"  # Could be bool or int
        try:
            int(first)
            return "IntArray"
        except ValueError:
            pass
        try:
            float(first)
            return "FloatArray"
        except ValueError:
            pass
        return "StringArray"
    
    # Check for nested records
    if value.startswith('{'):
        return "Record"
    
    # Check for booleans
    if value in ('0', '1', 'true', 'false', 'True', 'False'):
        return "Bool"  # Ambiguous with Int
    
    # Check for integers
    try:
        int(value)
        return "Int"
    except ValueError:
        pass
    
    # Check for floats
    try:
        float(value)
        return "Float"
    except ValueError:
        pass
    
    # Default to string
    return "String"

def validate_file(filepath: Path, registry: Dict[int, dict]) -> Optional[List[str]]:
    """Validate a single .lnmp file. Returns None for binary files."""
    errors = []
    
    try:
        # Try to read as text
        raw_bytes = filepath.read_bytes()
        
        # Check for binary content (null bytes or non-UTF8)
        if b'\x00' in raw_bytes:
            return None  # Binary file, skip
        
        try:
            content = raw_bytes.decode('utf-8')
        except UnicodeDecodeError:
            return None  # Binary file, skip
            
    except Exception as e:
        return [f"Could not read file: {e}"]
    
    # Extract FIDs
    fids = extract_fids_from_lnmp(content)
    
    for fid, value in fids:
        # Private range (32768-65535) is always valid
        if fid >= 32768:
            continue
        
        # Check if FID is defined
        if fid not in registry:
            # Extended range (16384-32767) might not be defined
            if fid >= 16384:
                continue  # Skip extended range
            errors.append(f"F{fid}: Unknown FID (not in registry)")
            continue
        
        # Check type match
        expected = registry[fid].get("type", "Unknown")
        inferred = infer_type(value)
        
        # Type compatibility check
        if not types_compatible(expected, inferred):
            errors.append(
                f"F{fid}: Type mismatch - expected {expected}, found {inferred} (value: {value[:30]}...)"
            )
    
    return errors

def types_compatible(expected: str, inferred: str) -> bool:
    """Check if inferred type is compatible with expected type."""
    if expected == inferred:
        return True
    
    # Bool/Int are often interchangeable in values like 0/1
    if expected in ("Bool", "Int") and inferred in ("Bool", "Int"):
        return True
    
    # IntArray/BoolArray can look similar with 0/1 values
    if expected in ("BoolArray", "IntArray") and inferred in ("BoolArray", "IntArray"):
        return True
    
    return False

def main():
    print("=" * 60)
    print("LNMP Fixture Validation")
    print("=" * 60)
    
    # Load registry
    registry = load_registry()
    print(f"\nLoaded {len(registry)} FID definitions from registry")
    
    # Find all .lnmp files
    spec_dir = Path(__file__).parent.parent / "spec" / "examples" / "text"
    fixtures_dir = Path(__file__).parent.parent / "fixtures"
    
    files = list(spec_dir.glob("*.lnmp")) + list(fixtures_dir.glob("*.lnmp"))
    print(f"Found {len(files)} .lnmp files\n")
    
    total_errors = 0
    validated = 0
    skipped = 0
    
    for filepath in sorted(files):
        errors = validate_file(filepath, registry)
        
        if errors is None:
            skipped += 1
            continue
        
        validated += 1
        
        if errors:
            print(f"❌ {filepath.name}:")
            for error in errors:
                print(f"   - {error}")
            total_errors += len(errors)
        else:
            print(f"✓ {filepath.name}")
    
    print("\n" + "=" * 60)
    print(f"Validated: {validated} files")
    print(f"Skipped (binary): {skipped} files")
    print(f"Total errors: {total_errors}")
    
    if total_errors > 0:
        print("\n⚠️  Some fixtures have FID registry issues!")
        return 1
    else:
        print("\n✅ All fixtures pass FID registry validation!")
        return 0

if __name__ == "__main__":
    sys.exit(main())
