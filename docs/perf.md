# Performance Guide for Janus

This document provides guidance on profiling, benchmarking, and optimizing Janus for maximum performance.

## Performance Goals

Janus is designed with performance as a first-class concern:

- **Hashing throughput**: 2-3 GB/s per core with BLAKE3
- **Directory walking**: 10,000+ files/second
- **Sync throughput**: I/O bound (500 MB/s - 3 GB/s on modern SSDs)
- **Memory usage**: Constant, regardless of file sizes
- **CPU utilization**: Scales linearly with available cores

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench hashing
cargo bench directory_walk

# Create baseline for comparison
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# View HTML reports
open target/criterion/report/index.html
```

### Interpreting Results

Criterion provides detailed statistics:

- **Time**: Mean execution time with confidence intervals
- **Throughput**: Operations or bytes per second
- **Change**: Percentage change from baseline (if available)
- **Outliers**: Identifies outlier measurements

Look for:
- Consistent throughput across runs (low variance)
- Linear scaling with data size (for streaming operations)
- No unexpected performance cliffs

### Writing Benchmarks

When adding new benchmarks:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_new_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("new_feature");
    
    // Set throughput for bytes/sec reporting
    group.throughput(Throughput::Bytes(1024 * 1024));
    
    group.bench_function("1MB", |b| {
        let data = vec![0u8; 1024 * 1024];
        b.iter(|| {
            // Use black_box to prevent compiler optimizations
            let result = process(black_box(&data));
            black_box(result);
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_new_feature);
criterion_main!(benches);
```

## Profiling

### Linux - perf

**Installation:**
```bash
# Ubuntu/Debian
sudo apt-get install linux-tools-common linux-tools-generic

# Arch Linux
sudo pacman -S perf
```

**Usage:**
```bash
# Build release binary
cargo build --release

# Record performance data
perf record --call-graph=dwarf ./target/release/janus scan /large/directory

# View report
perf report

# Generate flamegraph
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg
```

**What to look for:**
- Hot functions (high % in report)
- Excessive syscalls
- Cache misses (use `perf stat -d`)
- Branch mispredictions

### Linux - Flamegraph

**Installation:**
```bash
cargo install flamegraph
```

**Usage:**
```bash
# Requires root for perf_event_open
sudo cargo flamegraph -- scan /large/directory

# Open flamegraph.svg in browser
firefox flamegraph.svg
```

**Reading flamegraphs:**
- Width = time spent in function
- Height = call stack depth
- Color = usually arbitrary (some tools use color for module/language)
- Look for wide plateaus = optimization opportunities

### macOS - Instruments

**Usage:**
```bash
# Build release binary
cargo build --release

# Profile with Time Profiler
xcrun xctrace record --template 'Time Profiler' \
  --launch ./target/release/janus scan /large/directory

# Profile with Allocations
xcrun xctrace record --template 'Allocations' \
  --launch ./target/release/janus scan /large/directory
```

**Viewing results:**
```bash
# Open most recent trace
open $(ls -t ~/Library/Developer/Xcode/DerivedData/*.trace | head -1)
```

**What to look for:**
- CPU hotspots in Time Profiler
- Allocation patterns in Allocations instrument
- Excessive retain/release in ARC
- I/O patterns in System Trace

### macOS - Sample

Quick CPU profiling:
```bash
cargo build --release
sample ./target/release/janus 10 -file profile.txt
```

### Windows - VTune / PerfView

**VTune (Intel):**
```bash
# Hotspots analysis
vtune -collect hotspots -- target\release\janus.exe scan C:\large\directory

# View report
vtune -report hotspots
```

**PerfView (General):**
```bash
# Download from https://github.com/microsoft/perfview
PerfView.exe /DataFile:janus.etl run target\release\janus.exe scan C:\large\directory
```

### Valgrind (Linux) - Memory Profiling

**Massif (heap profiler):**
```bash
valgrind --tool=massif --massif-out-file=massif.out \
  ./target/release/janus scan /test/directory

# View results
ms_print massif.out
```

**Cachegrind (cache profiler):**
```bash
valgrind --tool=cachegrind ./target/release/janus scan /test/directory

# View results
cg_annotate cachegrind.out.<pid>
```

## Optimization Techniques

### 1. BLAKE3 vs SHA-256

**Performance comparison:**
```rust
// BLAKE3: ~10 GB/s on modern CPUs
// SHA-256: ~500 MB/s on modern CPUs

// Enable SHA-256 for compatibility (slower)
cargo build --release --features sha256
```

**When to use SHA-256:**
- Interoperability with existing systems using SHA-256
- Regulatory/compliance requirements
- Otherwise, prefer BLAKE3

### 2. Thread Pool Tuning

```bash
# Use fewer threads (reduce contention)
janus sync source/ dest/ -j 4

# Use more threads (if CPU-bound)
janus sync source/ dest/ -j 16

# Auto-detect (default)
janus sync source/ dest/
```

**Guidelines:**
- For SSDs: threads = CPU cores
- For HDDs: threads = 1-2 (seeks dominate)
- For network drives: experiment, usually 4-8

### 3. Buffer Size Tuning

Current buffer size: **64 KB**

This is tuned for modern SSDs. If you need to modify:

```rust
// In src/io.rs
const COPY_BUFFER_SIZE: usize = 64 * 1024;  // Current

// For older HDDs (reduce seeks)
const COPY_BUFFER_SIZE: usize = 128 * 1024;

// For very fast NVMe (reduce syscall overhead)
const COPY_BUFFER_SIZE: usize = 256 * 1024;
```

**Benchmark before/after:**
```bash
cargo bench copy_large_file
```

### 4. Parallel I/O (TODO)

Currently, file copies are sequential. For network drives or slow disks, parallel copying may help:

```rust
// Future optimization
files_to_copy.par_iter().try_for_each(|file| {
    copy_file_with_metadata(&source, &dest, true)
})?;
```

**Trade-off:** More contention on disk, but better network utilization.

### 5. Memory Pool for Buffers

Currently, buffers are allocated per operation. For very high file counts, consider pooling:

```rust
use crossbeam_channel::unbounded;

// Buffer pool
let (tx, rx) = unbounded();
for _ in 0..num_threads {
    tx.send(vec![0u8; BUFFER_SIZE]).unwrap();
}

// Acquire buffer
let mut buffer = rx.recv().unwrap();
// ... use buffer ...
// Return buffer
tx.send(buffer).unwrap();
```

### 6. Hash Caching (TODO)

For incremental syncs, cache hashes based on (path, size, mtime):

```rust
// Pseudo-code for future optimization
struct HashCache {
    cache: HashMap<(PathBuf, u64, SystemTime), ContentHash>,
}

impl HashCache {
    fn get_or_compute(&mut self, file: &FileMeta) -> io::Result<ContentHash> {
        let key = (file.path.clone(), file.size, file.mtime);
        if let Some(hash) = self.cache.get(&key) {
            return Ok(hash.clone());
        }
        // Compute and cache
        let hash = hash_file(&file.path)?;
        self.cache.insert(key, hash.clone());
        Ok(hash)
    }
}
```

**Performance gain:** Skip re-hashing unchanged files (huge for incremental syncs).

## Performance Monitoring

### Measuring Throughput

```bash
# Scan a known-size directory
time janus scan /path/to/1GB/of/files

# Calculate throughput
# If completed in 0.5 seconds: 1GB / 0.5s = 2 GB/s
```

### Tracking Regression

```bash
# Before changes
cargo bench -- --save-baseline before

# Make changes
# ... edit code ...

# After changes
cargo bench -- --baseline before

# Criterion will show performance delta
```

### CI Performance Tracking

Add to CI:
```yaml
- name: Run benchmarks
  run: cargo bench -- --save-baseline ci-${{ github.sha }}

- name: Upload benchmark results
  uses: actions/upload-artifact@v3
  with:
    name: benchmarks-${{ github.sha }}
    path: target/criterion/
```

## Debugging Performance Issues

### Symptom: Low CPU Usage

**Possible causes:**
- I/O bound (disk/network is bottleneck)
- Lock contention
- Not enough parallelism

**Debug:**
```bash
# Monitor I/O
iostat -x 1

# Check lock contention
perf record -e lock:contention_begin ./target/release/janus scan /dir
```

### Symptom: High Memory Usage

**Possible causes:**
- Memory leaks
- Excessive buffering
- Not using streaming

**Debug:**
```bash
# Valgrind massif
valgrind --tool=massif ./target/release/janus scan /dir

# Track allocations
perf record -e 'syscalls:sys_enter_mmap' ./target/release/janus scan /dir
```

### Symptom: Slow Hashing

**Possible causes:**
- Using SHA-256 instead of BLAKE3
- Small buffer size
- Disk I/O bottleneck

**Debug:**
```bash
# Check which hash algorithm
janus scan /dir | grep -i "blake\|sha"

# Benchmark hashing specifically
cargo bench hash_file

# Monitor disk I/O
sudo iotop -o
```

### Symptom: Slow Directory Walking

**Possible causes:**
- Network filesystem
- Many small files
- .gitignore processing overhead

**Debug:**
```bash
# Benchmark directory walk
cargo bench directory_walk

# Disable .gitignore processing
# (edit src/core.rs, set git_ignore(false))
```

## Best Practices

1. **Always benchmark before optimizing**: Use data, not intuition
2. **Profile in release mode**: Debug builds are 10-100x slower
3. **Use representative workloads**: Test with real-world data sizes
4. **Focus on hot paths**: Optimize the 20% of code that takes 80% of time
5. **Measure memory allocations**: Use `cargo-flamegraph --dev` with allocator profiling
6. **Test on target hardware**: Performance varies across systems
7. **Document optimizations**: Explain why, not just what
8. **Add benchmarks for critical paths**: Prevent regressions

## Further Reading

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [BLAKE3 Specifications](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Linux perf Tutorial](https://perf.wiki.kernel.org/index.php/Tutorial)
- [Brendan Gregg's Flamegraph Guide](http://www.brendangregg.com/flamegraphs.html)

## Performance Tuning Checklist

- [ ] Profile with representative workload
- [ ] Identify hot functions (>5% of time)
- [ ] Check for unnecessary allocations
- [ ] Verify parallel execution scales
- [ ] Benchmark before and after changes
- [ ] Test on different hardware configurations
- [ ] Document performance characteristics
- [ ] Add regression tests (benchmarks)

---

**Remember**: Premature optimization is the root of all evil. Profile first, then optimize.