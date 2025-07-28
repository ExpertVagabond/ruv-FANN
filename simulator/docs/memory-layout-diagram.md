# Neuro-Synaptic Chip Memory Layout Diagram

## 28MB Shared Memory Detailed Partition Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         28MB SHARED MEMORY POOL                              │
│                    Total: 29,360,128 bytes (0x1C00000)                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      MODEL WEIGHTS REGION (16MB)                     │   │
│  │                   Address: 0x0000000 - 0x0FFFFFF                     │   │
│  │                      16,777,216 bytes total                          │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │ Layer 1 Weights  │ Layer 2 Weights  │ ... │ Layer N Weights        │   │
│  │ (Variable size)  │ (Variable size)  │     │ (Variable size)        │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    ACTIVATION MEMORY REGION (8MB)                    │   │
│  │                   Address: 0x1000000 - 0x17FFFFF                     │   │
│  │                      8,388,608 bytes total                           │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │  Core 0  │  Core 1  │  Core 2  │ ... │ Core 254 │ Core 255        │   │
│  │  (32KB)  │  (32KB)  │  (32KB)  │     │  (32KB)  │  (32KB)         │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                      I/O BUFFER REGION (4MB)                         │   │
│  │                   Address: 0x1800000 - 0x1BFFFFF                     │   │
│  │                      4,194,304 bytes total                           │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │         INPUT BUFFERS (2MB)        │       OUTPUT BUFFERS (2MB)     │   │
│  │    Address: 0x1800000 - 0x19FFFFF  │  Address: 0x1A00000 - 0x1BFFFFF│   │
│  ├──────────────┬──────────────────────┼────────────┬──────────────────┤   │
│  │ Core 0 (8KB) │ Core 1-255 (8KB ea) │ Core 0 (8KB)│ Core 1-255 (8KB)│   │
│  └──────────────┴──────────────────────┴────────────┴──────────────────┘   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Detailed Memory Allocation per Core

### Per-Core Memory Breakdown
```
Each Core (0-255) has access to:
┌─────────────────────────────────────┐
│         SHARED (Read-Only)           │
│    Model Weights: 16MB (shared)      │
├─────────────────────────────────────┤
│        EXCLUSIVE (Read/Write)        │
│    Activation Memory: 32KB           │
│    Input Buffer: 8KB                 │
│    Output Buffer: 8KB                │
├─────────────────────────────────────┤
│    Total Exclusive: 48KB per core   │
│    Total Shared: 16MB               │
└─────────────────────────────────────┘
```

### Memory Address Calculation

```
For Core ID = N (where N = 0 to 255):

Activation Memory Address:
  Start: 0x1000000 + (N × 0x8000)    // 0x8000 = 32KB
  End:   Start + 0x7FFF

Input Buffer Address:
  Start: 0x1800000 + (N × 0x2000)    // 0x2000 = 8KB
  End:   Start + 0x1FFF

Output Buffer Address:
  Start: 0x1A00000 + (N × 0x2000)    // 0x2000 = 8KB
  End:   Start + 0x1FFF
```

## Memory Access Patterns

### Concurrent Access Map
```
Time →
T0: ┌─────────────────────────────────────────────────┐
    │ R R R R R R R R R R R R R R R R R R R R R R ... │ All 256 cores reading weights
    │ Core 0-255 reading from Weights Region          │
    └─────────────────────────────────────────────────┘

T1: ┌─────────────────────────────────────────────────┐
    │ W . . . │ . W . . │ . . W . │ ... │ . . . W     │ Each core writing to its
    │ Core 0   │ Core 1  │ Core 2  │     │ Core 255   │ exclusive activation region
    └─────────────────────────────────────────────────┘

T2: ┌─────────────────────────────────────────────────┐
    │ ═══════════════ BARRIER SYNC ═══════════════    │ All cores wait at barrier
    └─────────────────────────────────────────────────┘

Legend: R = Read, W = Write, . = Idle, ═ = Barrier
```

## WASM Memory Mapping

### WebAssembly Linear Memory View
```
WASM Linear Memory (28MB total)
┌────────────────────────────────────────┐
│ WASM Pages: 0-447 (448 pages total)    │
│ Each page: 65,536 bytes (64KB)         │
├────────────────────────────────────────┤
│ Pages 0-255:   Model Weights           │
│ Pages 256-383: Activations             │
│ Pages 384-447: I/O Buffers             │
└────────────────────────────────────────┘
```

### Memory Safety Zones
```
┌─────────────────────────────────────────────────────┐
│                  GUARD REGIONS                       │
├─────────────────────────────────────────────────────┤
│ Before Weights:     N/A (start of memory)           │
│ After Weights:      64-byte guard                   │
│ Between Cores:      64-byte guard per core          │
│ After I/O:          64-byte guard                   │
└─────────────────────────────────────────────────────┘
```

## Memory Bandwidth Utilization

### Peak Bandwidth Scenarios
```
Scenario 1: Layer Forward Pass
- All 256 cores read weights simultaneously
- Bandwidth: 256 × 64 bytes/cycle = 16KB/cycle
- Duration: Weight size / 16KB

Scenario 2: Activation Update
- Each core writes to exclusive region
- No contention, full parallelism
- Bandwidth: 256 × 32 bytes/cycle = 8KB/cycle

Scenario 3: Result Collection
- Sequential reads from output buffers
- Bandwidth: 64 bytes/cycle (serialized)
- Duration: 256 × 8KB / 64 bytes
```

## Memory Optimization Strategies

### 1. Cache Line Alignment
```
All memory regions aligned to 64-byte boundaries:
- Weight tensors: Start at 64-byte boundary
- Activation buffers: 32KB regions are 512 cache lines
- I/O buffers: 8KB regions are 128 cache lines
```

### 2. Memory Access Patterns for Optimal Performance
```
Good Pattern (Coalesced):          Bad Pattern (Strided):
Core 0: Read 0x0000-0x003F        Core 0: Read 0x0000
Core 1: Read 0x0040-0x007F        Core 1: Read 0x0100
Core 2: Read 0x0080-0x00BF        Core 2: Read 0x0200
→ Sequential, cache-friendly       → Cache misses, poor locality
```

### 3. Double Buffering for Pipelining
```
While cores process Layer N:
┌─────────────────────────┬─────────────────────────┐
│   Buffer A (Active)     │   Buffer B (Loading)    │
│   Processing Layer N    │   Prefetch Layer N+1    │
└─────────────────────────┴─────────────────────────┘
```

## Memory Constraints Enforcement

### Hard Limits
- Total memory: Cannot exceed 28MB (29,360,128 bytes)
- Per-core activation: Cannot exceed 32KB
- No dynamic allocation beyond initial setup
- WASM memory.grow() operations will fail

### Soft Limits (Warnings)
- Weight region > 80% full (performance degradation)
- Activation region > 90% full (risk of overflow)
- I/O buffers > 75% full (may limit batch size)