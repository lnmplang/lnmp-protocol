"""
LNMP Python SDK Benchmark Suite

Measures performance of critical SDK operations:
1. Core: Parse, Encode, Binary Encode/Decode
2. Spatial: Encode/Decode
3. Embedding: Delta computation

Usage:
    python benchmarks/run_benchmarks.py
"""

import timeit
import statistics
import lnmp
import os
import sys

# Ensure we can import lnmp if running from sdk/python root
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

def run_benchmark(name, func, setup_func=None, iterations=10000):
    """Run a single benchmark and return stats."""
    print(f"Running {name} ({iterations} iterations)...", end="", flush=True)
    
    setup_code = "pass"
    if setup_func:
        setup_code = setup_func()
        
    # Warmup
    timeit.timeit(func, setup=setup_code, number=iterations // 10, globals=globals())
    
    # Measurement
    times = timeit.repeat(func, setup=setup_code, repeat=5, number=iterations, globals=globals())
    
    # Calculate stats (time per operation in microseconds)
    avg_time_us = (statistics.mean(times) / iterations) * 1_000_000
    min_time_us = (min(times) / iterations) * 1_000_000
    max_time_us = (max(times) / iterations) * 1_000_000
    ops_per_sec = 1_000_000 / avg_time_us
    
    print(f" Done. {avg_time_us:.2f} µs/op ({int(ops_per_sec):,} ops/sec)")
    
    return {
        "name": name,
        "avg_us": avg_time_us,
        "min_us": min_time_us,
        "max_us": max_time_us,
        "ops_sec": ops_per_sec
    }

def main():
    results = []
    
    # --- Core Benchmarks ---
    
    # 1. Parse
    results.append(run_benchmark(
        "Core: Parse (Small)",
        lambda: lnmp.core.parse("F12=14532;F7=1"),
        iterations=50000
    ))
    
    # 2. Encode (Text)
    record = lnmp.core.parse("F12=14532;F7=1")
    results.append(run_benchmark(
        "Core: Encode Text (Small)",
        lambda: record.encode(),
        iterations=50000
    ))
    
    # 3. Encode (Binary)
    results.append(run_benchmark(
        "Core: Encode Binary (Small)",
        lambda: record.encode_binary(),
        iterations=50000
    ))
    
    # 4. Decode (Binary)
    binary_data = record.encode_binary()
    results.append(run_benchmark(
        "Core: Decode Binary (Small)",
        lambda: lnmp.core.decode_binary(binary_data),
        iterations=50000
    ))
    
    # --- Spatial Benchmarks ---
    
    results.append(run_benchmark(
        "Spatial: Encode Position3D",
        lambda: lnmp.spatial.encode_position3d(123.456, 789.012, 345.678),
        iterations=100000
    ))
    
    encoded_spatial = lnmp.spatial.encode_position3d(123.456, 789.012, 345.678)
    results.append(run_benchmark(
        "Spatial: Decode Position3D",
        lambda: lnmp.spatial.decode_position3d(encoded_spatial),
        iterations=100000
    ))
    
    # --- Embedding Benchmarks ---
    
    base_vec = [0.1 * i for i in range(128)]
    updated_vec = [0.1 * i + (0.01 if i % 10 == 0 else 0) for i in range(128)]
    
    results.append(run_benchmark(
        "Embedding: Delta (128-dim)",
        lambda: lnmp.embedding.delta(base_vec, updated_vec),
        iterations=10000
    ))
    
    # --- Report Generation ---
    
    report_path = os.path.join(os.path.dirname(__file__), "..", "BENCHMARKS.md")
    with open(report_path, "w") as f:
        f.write("# LNMP Python SDK Performance Benchmarks\n\n")
        f.write(f"**Date:** {timeit.time.strftime('%Y-%m-%d')}\n")
        f.write(f"**Platform:** {sys.platform}\n\n")
        f.write("| Operation | Avg Time (µs) | Ops/Sec | Min (µs) | Max (µs) |\n")
        f.write("|-----------|---------------|---------|----------|----------|\n")
        
        for r in results:
            f.write(f"| {r['name']} | {r['avg_us']:.2f} | {int(r['ops_sec']):,} | {r['min_us']:.2f} | {r['max_us']:.2f} |\n")
            
    print(f"\nBenchmark report generated at: {os.path.abspath(report_path)}")

if __name__ == "__main__":
    main()
