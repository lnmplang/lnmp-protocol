#!/usr/bin/env python3
"""
LNMP Registry Analyzer
Analyzes and validates the binary registry.lnmp file.
Implements a minimal LNMP v0.5 binary decoder.
"""

import sys
import struct
import datetime

# --- LNMP Constants ---
LNMP_MAGIC = b"LNMP"
TYPE_INT = 0x01
TYPE_STRING = 0x04
TYPE_NESTED_RECORD = 0x06
TYPE_NESTED_ARRAY = 0x07

# Registry FIDs
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
FID_ENTRY_STATUS = 4 # 0=Proposed, 1=Active, 2=Deprecated, 3=Tombstoned
FID_ENTRY_SINCE = 5

TYPE_MAP_REV = {
    1: "Int", 2: "Float", 3: "Bool", 4: "String",
    5: "StringArray", 6: "Record", 7: "RecordArray",
    8: "Embedding", 11: "IntArray", 12: "FloatArray", 13: "BoolArray"
}

STATUS_MAP_REV = {
    0: "PROPOSED", 1: "ACTIVE", 2: "DEPRECATED", 3: "TOMBSTONED"
}

class Decoder:
    def __init__(self, data):
        self.data = data
        self.pos = 0

    def read_byte(self):
        b = self.data[self.pos]
        self.pos += 1
        return b

    def read_u64_varint(self):
        result = 0
        shift = 0
        while True:
            byte = self.read_byte()
            result |= (byte & 0x7F) << shift
            if not (byte & 0x80):
                break
            shift += 7
        return result

    def read_i64_varint(self):
        raw = self.read_u64_varint()
        # ZigZag decode
        return (raw >> 1) ^ -(raw & 1)

    def read_bytes(self, length):
        data = self.data[self.pos:self.pos+length]
        self.pos += length
        return data

    def read_string(self):
        length = self.read_u64_varint()
        data = self.read_bytes(length)
        return data.decode('utf-8')

    def msg(self, fid, type_tag):
        # Value decoder based on type
        if type_tag == TYPE_INT:
            return self.read_i64_varint()
        elif type_tag == TYPE_STRING:
            return self.read_string()
        elif type_tag == TYPE_NESTED_RECORD:
            # Nested Record: [ValueContent] -> [FieldCount] [Fields...]
            # But wait, in our compiler we wrote: Header(FID, NESTED_RECORD) then Content.
            # Content was: [FieldCount] [Fields]
            # So we just recursively parse.
            return self.read_record_body()
        elif type_tag == TYPE_NESTED_ARRAY:
            # Nested Array: [Count] [Record1Body] [Record2Body]...
            count = self.read_u64_varint()
            records = []
            for _ in range(count):
                records.append(self.read_record_body())
            return records
        else:
            return f"<Unknown Type: {type_tag}>"

    def read_record_body(self):
        field_count = self.read_u64_varint()
        fields = {}
        for _ in range(field_count):
            # Read Field Header: [FID: u16] [TypeTag: u8]
            fid_bytes = self.read_bytes(2)
            fid = struct.unpack('<H', fid_bytes)[0]
            type_tag = self.read_byte()
            
            value = self.msg(fid, type_tag)
            fields[fid] = value
        return fields

def analyze_registry(path):
    print(f"Opening {path}...")
    try:
        with open(path, "rb") as f:
            data = f.read()
    except Exception as e:
        print(f"Error reading file: {e}")
        return

    print(f"File Size: {len(data)} bytes")
    
    decoder = Decoder(data)
    
    # 1. Check Header (12 bytes)
    if len(data) < 12:
        print("Error: File too short for LNMP header")
        return

    magic = decoder.read_bytes(4)
    if magic != LNMP_MAGIC:
        print(f"Error: Invalid Magic {magic}")
        return
        
    version = decoder.read_byte()
    mode = decoder.read_byte()
    
    # Flags: u16 BE
    flags_bytes = decoder.read_bytes(2)
    flags = struct.unpack('>H', flags_bytes)[0]
    
    # MetaLen: u32 BE
    meta_len_bytes = decoder.read_bytes(4)
    meta_len = struct.unpack('>I', meta_len_bytes)[0]
    
    print(f"Container Version: {version}")
    print(f"Mode: 0x{mode:02X} (Binary={0x02})")
    print(f"Flags: 0x{flags:04X}")
    print(f"Metadata Length: {meta_len}")
    print("-" * 40)
    
    # Skip metadata if any (parsed based on mode, but here we assume none or skip)
    if meta_len > 0:
        decoder.read_bytes(meta_len)
        print(f"Skipped {meta_len} bytes of metadata")
    
    # 2. Parse Root Record
    # The rest of the file is the Root Record Body
    try:
        root = decoder.read_record_body()
    except Exception as e:
        print(f"Error parsing root record: {e}")
        import traceback
        traceback.print_exc()
        return

    # 3. Analyze Content
    if FID_META in root:
        meta = root[FID_META]
        print("METADATA:")
        if FID_VERSION in meta:
            print(f"  Registry Version: {meta[FID_VERSION]}")
        if FID_PROTOCOL_VERSION in meta:
            print(f"  Protocol Version: {meta[FID_PROTOCOL_VERSION]}")
        if FID_GENERATED_AT in meta:
            ts = meta[FID_GENERATED_AT]
            dt = datetime.datetime.fromtimestamp(ts)
            print(f"  Generated At: {dt} (Timestamp: {ts})")
    else:
        print("Warning: No Metadata found!")

    print("-" * 40)

    if FID_FIDS in root:
        fids = root[FID_FIDS]
        print(f"TOTAL FIDS: {len(fids)}")
        
        # Stats
        types = {}
        statuses = {}
        
        print("\nSAMPLE FIDS:")
        for i, entry in enumerate(fids):
            fid = entry.get(FID_ENTRY_ID, -1)
            name = entry.get(FID_ENTRY_NAME, "???")
            type_val = entry.get(FID_ENTRY_TYPE, 0)
            status_val = entry.get(FID_ENTRY_STATUS, 0)
            since = entry.get(FID_ENTRY_SINCE, "")
            
            type_str = TYPE_MAP_REV.get(type_val, f"Unknown({type_val})")
            status_str = STATUS_MAP_REV.get(status_val, f"Unknown({status_val})")
            
            # Update stats
            types[type_str] = types.get(type_str, 0) + 1
            statuses[status_str] = statuses.get(status_str, 0) + 1
            
            # Print first 5 and specific ones
            if i < 5 or fid in [256, 512, 1024]:
                print(f"  [{fid}] {name:<20} Type: {type_str:<10} Status: {status_str:<10} Since: {since}")
            if i == 5:
                print("  ...")

        print("\nSTATISTICS:")
        print("  By Type:")
        for t, c in types.items():
            print(f"    - {t}: {c}")
            
        print("  By Status:")
        for s, c in statuses.items():
            print(f"    - {s}: {c}")

    else:
        print("Warning: No FIDs found!")

if __name__ == "__main__":
    analyze_registry("registry/registry.lnmp")
