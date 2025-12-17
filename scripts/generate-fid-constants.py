#!/usr/bin/env python3
"""
LNMP FID Constants Generator

Generates SDK-specific constant files from registry/fids.yaml.
Supports: Rust, TypeScript, Python, Go

Usage:
    python scripts/generate-fid-constants.py [--lang rust|ts|python|go|all]
"""

import argparse
from datetime import datetime
from pathlib import Path

try:
    import yaml
except ImportError:
    print("ERROR: Missing pyyaml. Run: pip install pyyaml")
    exit(1)


REGISTRY_PATH = Path(__file__).parent.parent / "registry" / "fids.yaml"
OUTPUT_DIR = Path(__file__).parent.parent / "generated"


def load_registry():
    """Load and parse the FID registry."""
    with open(REGISTRY_PATH) as f:
        return yaml.safe_load(f)


def get_all_active_fields(data):
    """Get all ACTIVE fields from all sections."""
    fields = []
    for section in ["core", "standard", "extended"]:
        if section in data and data[section]:
            for field in data[section]:
                if field.get("status") == "ACTIVE":
                    fields.append({**field, "_section": section})
    return sorted(fields, key=lambda x: x["fid"])


def to_upper_snake(name: str) -> str:
    """Convert snake_case to UPPER_SNAKE_CASE."""
    return name.upper()


def generate_rust(fields, metadata):
    """Generate Rust constants file."""
    lines = [
        "//! LNMP Field ID Constants",
        "//!",
        f"//! Auto-generated from registry/fids.yaml v{metadata['version']}",
        f"//! Generated: {datetime.now().isoformat()}",
        "//!",
        "//! DO NOT EDIT MANUALLY",
        "",
        "#![allow(dead_code)]",
        "",
        "/// Field ID type alias",
        "pub type Fid = u16;",
        "",
        "// =============================================================================",
        "// CORE FIELDS (0-255) - LOCKED",
        "// =============================================================================",
        "",
    ]

    current_section = "core"
    for field in fields:
        if field["_section"] != current_section:
            current_section = field["_section"]
            lines.extend([
                "",
                "// =============================================================================",
                f"// {current_section.upper()} FIELDS",
                "// =============================================================================",
                "",
            ])

        const_name = f"FID_{to_upper_snake(field['name'])}"
        desc = field.get("description", "")
        unit = field.get("unit")
        unit_str = f" (unit: {unit})" if unit else ""

        lines.append(f"/// F{field['fid']}: {desc}{unit_str}")
        lines.append(f"pub const {const_name}: Fid = {field['fid']};")
        lines.append("")

    return "\n".join(lines)


def generate_typescript(fields, metadata):
    """Generate TypeScript constants file."""
    lines = [
        "/**",
        " * LNMP Field ID Constants",
        " *",
        f" * Auto-generated from registry/fids.yaml v{metadata['version']}",
        f" * Generated: {datetime.now().isoformat()}",
        " *",
        " * DO NOT EDIT MANUALLY",
        " */",
        "",
        "/** Field ID type */",
        "export type Fid = number;",
        "",
        "/** LNMP Official Field IDs */",
        "export const FID = {",
    ]

    for field in fields:
        const_name = to_upper_snake(field["name"])
        desc = field.get("description", "")
        lines.append(f"  /** F{field['fid']}: {desc} */")
        lines.append(f"  {const_name}: {field['fid']},")

    lines.extend([
        "} as const;",
        "",
        "/** Type for FID keys */",
        "export type FidKey = keyof typeof FID;",
        "",
        "/** Reverse lookup: FID number to name */",
        "export const FID_NAMES: Record<number, string> = {",
    ])

    for field in fields:
        lines.append(f"  {field['fid']}: '{field['name']}',")

    lines.extend([
        "};",
        "",
    ])

    return "\n".join(lines)


def generate_python(fields, metadata):
    """Generate Python constants file."""
    lines = [
        '"""',
        "LNMP Field ID Constants",
        "",
        f"Auto-generated from registry/fids.yaml v{metadata['version']}",
        f"Generated: {datetime.now().isoformat()}",
        "",
        "DO NOT EDIT MANUALLY",
        '"""',
        "",
        "from typing import Dict",
        "",
        "",
        "class FID:",
        '    """LNMP Official Field IDs."""',
        "",
    ]

    for field in fields:
        const_name = to_upper_snake(field["name"])
        desc = field.get("description", "")
        lines.append(f"    # F{field['fid']}: {desc}")
        lines.append(f"    {const_name}: int = {field['fid']}")
        lines.append("")

    lines.extend([
        "",
        "# Reverse lookup: FID number to name",
        "FID_NAMES: Dict[int, str] = {",
    ])

    for field in fields:
        lines.append(f"    {field['fid']}: '{field['name']}',")

    lines.extend([
        "}",
        "",
    ])

    return "\n".join(lines)


def generate_go(fields, metadata):
    """Generate Go constants file."""
    lines = [
        "// Package fid provides LNMP Field ID constants.",
        "//",
        f"// Auto-generated from registry/fids.yaml v{metadata['version']}",
        f"// Generated: {datetime.now().isoformat()}",
        "//",
        "// DO NOT EDIT MANUALLY",
        "package fid",
        "",
        "// Fid is the Field ID type",
        "type Fid uint16",
        "",
        "// LNMP Official Field IDs",
        "const (",
    ]

    for field in fields:
        const_name = "Fid" + "".join(word.title() for word in field["name"].split("_"))
        desc = field.get("description", "")
        lines.append(f"\t// {const_name} - F{field['fid']}: {desc}")
        lines.append(f"\t{const_name} Fid = {field['fid']}")
        lines.append("")

    lines.extend([
        ")",
        "",
        "// FidNames maps FID numbers to names",
        "var FidNames = map[Fid]string{",
    ])

    for field in fields:
        const_name = "Fid" + "".join(word.title() for word in field["name"].split("_"))
        lines.append(f'\t{const_name}: "{field["name"]}",')

    lines.extend([
        "}",
        "",
    ])

    return "\n".join(lines)


GENERATORS = {
    "rust": ("fids.rs", generate_rust),
    "ts": ("fids.ts", generate_typescript),
    "python": ("fids.py", generate_python),
    "go": ("fids.go", generate_go),
}


def main():
    parser = argparse.ArgumentParser(description="Generate FID constants for SDKs")
    parser.add_argument(
        "--lang",
        choices=["rust", "ts", "python", "go", "all"],
        default="all",
        help="Target language (default: all)",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=OUTPUT_DIR,
        help=f"Output directory (default: {OUTPUT_DIR})",
    )
    args = parser.parse_args()

    print("=" * 60)
    print("LNMP FID Constants Generator")
    print("=" * 60)
    print()

    data = load_registry()
    metadata = data["metadata"]
    fields = get_all_active_fields(data)

    print(f"Registry version: {metadata['version']}")
    print(f"Protocol version: {metadata['protocol_version']}")
    print(f"Active fields: {len(fields)}")
    print()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    langs = GENERATORS.keys() if args.lang == "all" else [args.lang]

    for lang in langs:
        filename, generator = GENERATORS[lang]
        content = generator(fields, metadata)
        output_path = args.output_dir / filename

        with open(output_path, "w") as f:
            f.write(content)

        print(f"✓ Generated {output_path}")

    print()
    print("=" * 60)
    print("Done!")
    print("=" * 60)


if __name__ == "__main__":
    main()
