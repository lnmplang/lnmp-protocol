#!/usr/bin/env python3
"""
LNMP FID Breaking Change Detector

Detects breaking changes in registry/fids.yaml between commits.
A breaking change is:
1. Changing type of an ACTIVE field
2. Changing unit of an ACTIVE field
3. Changing name of an ACTIVE field
4. Removing an ACTIVE field (should be DEPRECATED first)
5. Reusing a TOMBSTONED FID
"""

import subprocess
import sys

try:
    import yaml
except ImportError:
    print("ERROR: Missing pyyaml. Run: pip install pyyaml")
    sys.exit(1)


REGISTRY_PATH = "registry/fids.yaml"


def get_file_at_ref(ref: str, path: str) -> str:
    """Get file content at a specific git ref."""
    try:
        result = subprocess.run(
            ["git", "show", f"{ref}:{path}"],
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout
    except subprocess.CalledProcessError:
        return None


def load_registry_at_ref(ref: str) -> dict:
    """Load registry from a git ref."""
    content = get_file_at_ref(ref, REGISTRY_PATH)
    if content is None:
        return None
    return yaml.safe_load(content)


def load_current_registry() -> dict:
    """Load current registry from working tree."""
    with open(REGISTRY_PATH) as f:
        return yaml.safe_load(f)


def get_all_fields(data: dict) -> dict:
    """Extract all fields as {fid: field_dict}."""
    fields = {}
    for section in ["core", "standard", "extended", "tombstoned"]:
        if section in data and data[section]:
            for field in data[section]:
                fields[field["fid"]] = {**field, "_section": section}
    return fields


def detect_breaking_changes(old_data: dict, new_data: dict) -> list:
    """Detect breaking changes between two registry versions."""
    if old_data is None:
        # No previous registry, nothing to compare
        return []

    old_fields = get_all_fields(old_data)
    new_fields = get_all_fields(new_data)
    
    breaking_changes = []

    # Check each field in old registry
    for fid, old_field in old_fields.items():
        if old_field.get("status") not in ["ACTIVE", "DEPRECATED"]:
            continue

        if fid not in new_fields:
            # Field removed
            if old_field.get("status") == "ACTIVE":
                breaking_changes.append({
                    "type": "REMOVED",
                    "fid": fid,
                    "name": old_field["name"],
                    "message": f"ACTIVE field F{fid} ({old_field['name']}) was removed. Deprecate first!",
                })
            continue

        new_field = new_fields[fid]

        # Check type change
        if old_field.get("type") != new_field.get("type"):
            breaking_changes.append({
                "type": "TYPE_CHANGE",
                "fid": fid,
                "name": old_field["name"],
                "old": old_field.get("type"),
                "new": new_field.get("type"),
                "message": f"F{fid} type changed: {old_field.get('type')} → {new_field.get('type')}",
            })

        # Check unit change
        if old_field.get("unit") != new_field.get("unit"):
            breaking_changes.append({
                "type": "UNIT_CHANGE",
                "fid": fid,
                "name": old_field["name"],
                "old": old_field.get("unit"),
                "new": new_field.get("unit"),
                "message": f"F{fid} unit changed: {old_field.get('unit')} → {new_field.get('unit')}",
            })

        # Check name change (semantic)
        if old_field.get("name") != new_field.get("name"):
            breaking_changes.append({
                "type": "NAME_CHANGE",
                "fid": fid,
                "old": old_field["name"],
                "new": new_field["name"],
                "message": f"F{fid} name changed: {old_field['name']} → {new_field['name']}",
            })

    # Check for tombstoned FID reuse
    old_tombstoned = set()
    if "tombstoned" in old_data and old_data["tombstoned"]:
        old_tombstoned = {f["fid"] for f in old_data["tombstoned"]}

    for fid, new_field in new_fields.items():
        if fid in old_tombstoned and new_field["_section"] != "tombstoned":
            breaking_changes.append({
                "type": "TOMBSTONE_REUSE",
                "fid": fid,
                "name": new_field["name"],
                "message": f"TOMBSTONED F{fid} cannot be reused!",
            })

    return breaking_changes


def main():
    """Run breaking change detection."""
    # Get base ref from command line or use origin/main
    base_ref = sys.argv[1] if len(sys.argv) > 1 else "origin/main"

    print("=" * 60)
    print("LNMP FID Breaking Change Detection")
    print(f"Comparing against: {base_ref}")
    print("=" * 60)
    print()

    old_data = load_registry_at_ref(base_ref)
    new_data = load_current_registry()

    if old_data is None:
        print(f"Note: No registry found at {base_ref}, skipping comparison")
        print("✓ No breaking changes detected (no baseline)")
        sys.exit(0)

    breaking_changes = detect_breaking_changes(old_data, new_data)

    if not breaking_changes:
        print("✓ No breaking changes detected")
        sys.exit(0)

    print(f"✗ Found {len(breaking_changes)} breaking change(s):")
    print()

    for change in breaking_changes:
        print(f"  [{change['type']}] {change['message']}")

    print()
    print("=" * 60)
    print("BREAKING CHANGES DETECTED - PR BLOCKED")
    print()
    print("If you need to make these changes:")
    print("  1. Create a NEW FID for the new definition")
    print("  2. Mark the old FID as DEPRECATED")
    print("  3. After 2 minor versions, mark as TOMBSTONED")
    print("=" * 60)
    sys.exit(1)


if __name__ == "__main__":
    main()
