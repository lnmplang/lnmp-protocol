#!/usr/bin/env python3
"""
LNMP Registry Compiler
Compiles fids.yaml into a binary registry.lnmp file.
This script implements a minimal LNMP v0.5 binary encoder to avoid external dependencies.
"""

import sys
import yaml
import struct
import json
import time
from datetime import datetime
from enum import Enum, auto

# --- LNMP Constants ---
LNMP_MAGIC = b"LNMP"
LNMP_CONTAINER_VERSION = 1
TYPE_INT = 0x01
TYPE_STRING = 0x04
TYPE_NESTED_RECORD = 0x06
TYPE_NESTED_ARRAY = 0x07

# Registry FIDs (Self-describing)
FID_META = 1
FID_FIDS = 2

# Meta FIDs
FID_VERSION = 1
FID_PROTOCOL_VERSION = 2
FID_GENERATED_AT = 3

# FID Entry FIDs
FID_ENTRY_ID = 1
FID_ENTRY_NAME = 2
FID_ENTRY_TYPE = 3
FID_ENTRY_STATUS = 4
FID_ENTRY_SINCE = 5

# Enum Mappings
TYPE_MAP = {
    "Int": 1, "Float": 2, "Bool": 3, "String": 4,
    "StringArray": 5, "Record": 6, "RecordArray": 7,
    "Embedding": 8, "IntArray": 11, "FloatArray": 12, "BoolArray": 13
}

STATUS_MAP = {
    "PROPOSED": 0, "ACTIVE": 1, "DEPRECATED": 2, "TOMBSTONED": 3
}

class Encoder:
    def __init__(self):
        self.buffer = bytearray()

    def write_u64_varint(self, value):
        while value >= 0x80:
            self.buffer.append((value & 0x7F) | 0x80)
            value >>= 7
        self.buffer.append(value & 0x7F)

    def write_i64_varint(self, value):
        # ZigZag encoding
        encoded = (value << 1) ^ (value >> 63)
        self.write_u64_varint(encoded)

    def write_bytes(self, data):
        self.buffer.extend(data)

    def write_string(self, s):
        data = s.encode('utf-8')
        self.write_u64_varint(len(data))
        self.write_bytes(data)

    def write_header(self, fid, type_tag):
        # Write FID (u16 -> VarInt per v0.5 spec? No, FID is u16 LE in spec or VarInt?)
        # Checking spec: FID is u16 (2 bytes) in Record field header.
        # But wait, v0.5 spec says:
        # Field: [FID: u16] [Type: u8] [Value]
        # Let's verify with rust code or assumes standard u16 LE.
        # Rust codec: byteorder::LittleEndian::write_u16
        self.buffer.extend(struct.pack('<H', fid))
        self.buffer.append(type_tag)

    def encode_int_field(self, fid, value):
        self.write_header(fid, TYPE_INT)
        self.write_i64_varint(value)

    def encode_string_field(self, fid, value):
        self.write_header(fid, TYPE_STRING)
        self.write_string(value)

    def encode_nested_record_field(self, fid, record_bytes):
        self.write_header(fid, TYPE_NESTED_RECORD)
        # Nested record logic: [Field Count: VarInt] [Fields...]
        # But here record_bytes should already contain structure
        self.write_u64_varint(len(record_bytes)) # Length of nested data? 
        # Wait, v0.5 NestedRecord: [TypeTag] [FieldCount] ...
        # My write_header writes TypeTag.
        # So I need to write content.
        # The content of a NestedRecord value is: [FieldCount: VarInt] [Field1] [Field2]...
        self.write_bytes(record_bytes)

    def encode_nested_array_field(self, fid, records_bytes_list):
        self.write_header(fid, TYPE_NESTED_ARRAY)
        # NestedArray: [Count: VarInt] [Record1] [Record2] ...
        # Each Record in array: [FieldCount: VarInt] [Fields...]
        self.write_u64_varint(len(records_bytes_list))
        for rb in records_bytes_list:
            self.write_bytes(rb)

class RecordBuilder:
    def __init__(self):
        self.fields = [] # (fid, bytes)

    def add_int(self, fid, value):
        enc = Encoder()
        enc.encode_int_field(fid, value)
        self.fields.append(enc.buffer)

    def add_string(self, fid, value):
        enc = Encoder()
        enc.encode_string_field(fid, value)
        self.fields.append(enc.buffer)

    def add_record(self, fid, builder):
        # Build inner record content
        content = builder.build_content()
        
        enc = Encoder()
        # Custom writing for nested record field
        enc.write_header(fid, TYPE_NESTED_RECORD)
        enc.write_bytes(content)
        self.fields.append(enc.buffer)
        
    def add_record_array(self, fid, builders):
        enc = Encoder()
        enc.write_header(fid, TYPE_NESTED_ARRAY)
        enc.write_u64_varint(len(builders))
        for b in builders:
            enc.write_bytes(b.build_content())
        self.fields.append(enc.buffer)

    def build_content(self):
        # Record Content: [FieldCount: VarInt] [Field1] ...
        enc = Encoder()
        enc.write_u64_varint(len(self.fields))
        # Sort fields by FID for canonical ordering
        # Need to parse FID back to sort? Or just store (fid, bytes) tuple
        # But I stored bytes with header.
        # Let's fix add_*
        pass
        
    def get_bytes(self):
        buffer = bytearray()
        # Write Field Count
        enc = Encoder()
        enc.write_u64_varint(len(self.fields))
        buffer.extend(enc.buffer)
        
        # Write Fields (Assumed added in order or sorted)
        # For this script we will add in order or sort by FID
        # self.fields is list of bytearrays. 
        # Wait, I need to allow sorting.
        # Let's change self.fields to list of (fid, bytearray)
        for _, data in self.fields:
            buffer.extend(data)
        return buffer

# Redefine builder for sorting
class SimpleRecordBuilder:
    def __init__(self):
        self.fields = [] # (fid, bytearray)

    def _add(self, fid, raw_bytes):
        self.fields.append((fid, raw_bytes))

    def add_int(self, fid, value):
        enc = Encoder()
        enc.encode_int_field(fid, value)
        self._add(fid, enc.buffer)

    def add_string(self, fid, value):
        enc = Encoder()
        enc.encode_string_field(fid, value)
        self._add(fid, enc.buffer)

    def add_record(self, fid, builder):
        # Value of NestedRecord is: [FieldCount] [Fields]
        content = builder.build_body()
        enc = Encoder()
        enc.write_header(fid, TYPE_NESTED_RECORD)
        enc.write_bytes(content)
        self._add(fid, enc.buffer)

    def add_record_array(self, fid, builders):
        # Value of NestedArray is: [ElementCount] [Record1Body] [Record2Body]
        enc = Encoder()
        enc.write_header(fid, TYPE_NESTED_ARRAY)
        enc.write_u64_varint(len(builders))
        for b in builders:
            enc.write_bytes(b.build_body())
        self._add(fid, enc.buffer)

    def build_body(self):
        # Sort by FID
        self.fields.sort(key=lambda x: x[0])
        
        enc = Encoder()
        enc.write_u64_varint(len(self.fields))
        buf = enc.buffer
        for _, data in self.fields:
            buf.extend(data)
        return buf

def main():
    try:
        with open("registry/fids.yaml", "r") as f:
            data = yaml.safe_load(f)
    except Exception as e:
        print(f"Error reading YAML: {e}")
        sys.exit(1)

    # 1. Build Meta Record
    meta_builder = SimpleRecordBuilder()
    meta_builder.add_string(FID_VERSION, data["metadata"]["version"])
    meta_builder.add_string(FID_PROTOCOL_VERSION, data["metadata"]["protocol_version"])
    meta_builder.add_int(FID_GENERATED_AT, int(time.time()))

    # 2. Build FIDs Array
    fid_builders = []
    
    all_entries = []
    for section in ["core", "standard", "extended", "tombstoned"]:
        if section in data and data[section]:
            all_entries.extend(data[section])
            
    # Sort entries by FID
    all_entries.sort(key=lambda x: x["fid"])
    
    for entry in all_entries:
        fb = SimpleRecordBuilder()
        fb.add_int(FID_ENTRY_ID, entry["fid"])
        
        # Name (skip if removed/tombstoned? No, keep history if present)
        if "name" in entry:
            fb.add_string(FID_ENTRY_NAME, entry["name"])
            
        # Type
        if "type" in entry:
            type_val = TYPE_MAP.get(entry["type"], 0)
            fb.add_int(FID_ENTRY_TYPE, type_val)
            
        # Status
        if "status" in entry:
            status_val = STATUS_MAP.get(entry["status"], 0)
            fb.add_int(FID_ENTRY_STATUS, status_val)
            
        # Since
        if "since" in entry:
            fb.add_string(FID_ENTRY_SINCE, str(entry["since"]))
            
        fid_builders.append(fb)

    # 3. Build Root Record
    root_builder = SimpleRecordBuilder()
    root_builder.add_record(FID_META, meta_builder)
    root_builder.add_record_array(FID_FIDS, fid_builders)
    
    root_body = root_builder.build_body()
    
    # 4. Write Binary Container
    with open("registry/registry.lnmp", "wb") as f:
        # Header (12 bytes)
        # Magic (4) + Version (1) + Mode (1) + Flags (2, BE) + MetaLen (4, BE)
        # Mode 0x02 = Binary
        f.write(LNMP_MAGIC)
        f.write(struct.pack('B', LNMP_CONTAINER_VERSION)) # Version: u8
        f.write(struct.pack('B', 0x02))                  # Mode: u8 (Binary)
        f.write(struct.pack('>H', 0))                    # Flags: u16 BE
        f.write(struct.pack('>I', 0))                    # MetaLen: u32 BE
        f.write(root_body)

    # 5. Write Canonical Text Format (Readable LNMP)
    # Mapping back numeric types/status to strings for readability in text format
    text_data = {
        "meta": data["metadata"],
        "fids": []
    }
    
    # Add generated_at to meta
    text_data["meta"]["generated_at"] = int(time.time())

    # Re-process entries for cleaner text output
    for entry in all_entries:
        cleaned = entry.copy()
        # Ensure FID is first for readability
        ordered = {"fid": cleaned.pop("fid")}
        ordered.update(cleaned)
        text_data["fids"].append(ordered)

    with open("registry/registry.canonical.lnmp", "w") as f:
        f.write("# LNMP Registry Text Format\n")
        f.write("# Generated automatically. DO NOT EDIT.\n")
        # Use json dump as base for LNMP text format (compatible syntax)
        json.dump(text_data, f, indent=2)
        f.write("\n")
        
    print(f"Successfully compiled registry/registry.lnmp ({len(root_body) + 8} bytes)")
    print(f"Successfully generated registry/registry.canonical.lnmp (Text format)")
    print(f"Stats: {len(fid_builders)} FIDs compiled.")

if __name__ == "__main__":
    main()
