#!/usr/bin/env python3
"""
LNMP FID Registry Validator

Validates registry/fids.yaml against:
1. JSON Schema compliance
2. FID uniqueness
3. Range compliance
4. Naming conventions
5. Status consistency
"""

import json
import sys
from pathlib import Path

try:
    import yaml
    from jsonschema import validate, ValidationError
except ImportError:
    print("ERROR: Missing dependencies. Run: pip install pyyaml jsonschema")
    sys.exit(1)


REGISTRY_PATH = Path(__file__).parent.parent / "registry" / "fids.yaml"
SCHEMA_PATH = Path(__file__).parent.parent / "registry" / "schema.json"

# FID range definitions
RANGES = {
    "core": (0, 1023),
    "standard": (1024, 16383),
    "extended": (16384, 32767),
    "tombstoned": (0, 65535),  # Tombstoned can be from any range
}


def load_registry():
    """Load and parse the FID registry."""
    with open(REGISTRY_PATH) as f:
        return yaml.safe_load(f)


def load_schema():
    """Load the JSON schema."""
    with open(SCHEMA_PATH) as f:
        return json.load(f)


def validate_schema(data, schema):
    """Validate registry against JSON schema."""
    try:
        validate(instance=data, schema=schema)
        print("✓ Schema validation passed")
        return True
    except ValidationError as e:
        print(f"✗ Schema validation failed: {e.message}")
        print(f"  Path: {' -> '.join(str(p) for p in e.path)}")
        return False


def validate_uniqueness(data):
    """Check that all FIDs are unique across all sections."""
    all_fids = []
    fid_locations = {}

    for section in ["core", "standard", "extended", "tombstoned"]:
        if section in data and data[section]:
            for field in data[section]:
                fid = field["fid"]
                if fid in fid_locations:
                    print(f"✗ Duplicate FID {fid}: found in both '{fid_locations[fid]}' and '{section}'")
                    return False
                fid_locations[fid] = section
                all_fids.append(fid)

    print(f"✓ FID uniqueness check passed ({len(all_fids)} unique FIDs)")
    return True


def validate_ranges(data):
    """Check that FIDs are in correct ranges for their section."""
    errors = []

    for section, (min_fid, max_fid) in RANGES.items():
        if section not in data or not data[section]:
            continue

        for field in data[section]:
            fid = field["fid"]
            if section != "tombstoned" and (fid < min_fid or fid > max_fid):
                errors.append(f"FID {fid} ({field['name']}) in '{section}' should be {min_fid}-{max_fid}")

    if errors:
        print("✗ Range validation failed:")
        for error in errors:
            print(f"  - {error}")
        return False

    print("✓ Range validation passed")
    return True


def validate_names(data):
    """Check naming conventions."""
    import re
    pattern = re.compile(r"^[a-z][a-z0-9_]*$")
    errors = []

    for section in ["core", "standard", "extended", "tombstoned"]:
        if section not in data or not data[section]:
            continue

        for field in data[section]:
            name = field["name"]
            if not pattern.match(name):
                errors.append(f"FID {field['fid']}: name '{name}' must be snake_case")

    if errors:
        print("✗ Naming validation failed:")
        for error in errors:
            print(f"  - {error}")
        return False

    print("✓ Naming validation passed")
    return True


def validate_status_consistency(data):
    """Check status field consistency."""
    errors = []

    # Tombstoned fields must have TOMBSTONED status
    if "tombstoned" in data and data["tombstoned"]:
        for field in data["tombstoned"]:
            if field.get("status") != "TOMBSTONED":
                errors.append(f"FID {field['fid']} in 'tombstoned' must have status=TOMBSTONED")

    # DEPRECATED fields must have deprecated_since
    for section in ["core", "standard", "extended"]:
        if section not in data or not data[section]:
            continue

        for field in data[section]:
            if field.get("status") == "DEPRECATED" and not field.get("deprecated_since"):
                errors.append(f"FID {field['fid']}: DEPRECATED status requires 'deprecated_since'")

    if errors:
        print("✗ Status consistency check failed:")
        for error in errors:
            print(f"  - {error}")
        return False

    print("✓ Status consistency check passed")
    return True


def main():
    """Run all validations."""
    print("=" * 60)
    print("LNMP FID Registry Validation")
    print("=" * 60)
    print()

    if not REGISTRY_PATH.exists():
        print(f"ERROR: Registry not found at {REGISTRY_PATH}")
        sys.exit(1)

    if not SCHEMA_PATH.exists():
        print(f"ERROR: Schema not found at {SCHEMA_PATH}")
        sys.exit(1)

    data = load_registry()
    schema = load_schema()

    results = [
        validate_schema(data, schema),
        validate_uniqueness(data),
        validate_ranges(data),
        validate_names(data),
        validate_status_consistency(data),
    ]

    print()
    if all(results):
        print("=" * 60)
        print("✓ All validations passed!")
        print("=" * 60)
        sys.exit(0)
    else:
        print("=" * 60)
        print("✗ Validation failed!")
        print("=" * 60)
        sys.exit(1)


if __name__ == "__main__":
    main()
