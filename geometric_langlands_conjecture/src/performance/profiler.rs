//! Performance profiling and analysis
//!
//! This module provides detailed performance profiling capabilities
//! including timing, memory tracking, and bottleneck identification.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

/// Performance profiler
pub struct Profiler {
    /// Timing data
    timings: HashMap<String, Vec<TimingEntry>>,
    /// Memory snapshots
    memory_snapshots: Vec<MemorySnapshot>,
    /// Call graph
    call_graph: CallGraph,
    /// Start time
    start_time: Instant,
}

/// Single timing entry
#[derive(Debug, Clone)]
struct TimingEntry {
    /// Start time
    start: Instant,
    /// Duration
    duration: Duration,
    /// Memory at start
    memory_start: usize,
    /// Memory at end
    memory_end: usize,
}

/// Memory snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp
    pub timestamp: Duration,
    /// Total allocated
    pub allocated: usize,
    /// In use
    pub in_use: usize,
    /// Peak usage
    pub peak: usize,
}

/// Call graph for function relationships
#[derive(Debug, Clone)]
struct CallGraph {
    /// Nodes (functions)
    nodes: HashMap<String, CallNode>,
    /// Current call stack
    call_stack: Vec<String>,
}

/// Call graph node
#[derive(Debug, Clone)]
struct CallNode {
    /// Function name
    name: String,
    /// Total time
    total_time: Duration,
    /// Self time (excluding children)
    self_time: Duration,
    /// Call count
    call_count: u64,
    /// Children
    children: HashMap<String, u64>,
}

/// Profile report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileReport {
    /// Total runtime
    pub total_runtime: Duration,
    /// Function statistics
    pub functions: Vec<FunctionStats>,
    /// Hotspots (top time consumers)
    pub hotspots: Vec<Hotspot>,
    /// Memory statistics
    pub memory_stats: MemoryStats,
    /// Bottlenecks identified
    pub bottlenecks: Vec<Bottleneck>,
}

/// Function statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionStats {
    /// Function name
    pub name: String,
    /// Total time
    pub total_time: Duration,
    /// Average time
    pub avg_time: Duration,
    /// Min time
    pub min_time: Duration,
    /// Max time
    pub max_time: Duration,
    /// Call count
    pub call_count: u64,
    /// Time percentage
    pub time_percentage: f64,
}

/// Performance hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    /// Function name
    pub function: String,
    /// Time spent
    pub time: Duration,
    /// Percentage of total
    pub percentage: f64,
    /// Call location
    pub location: Option<String>,
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Peak memory usage
    pub peak_usage: usize,
    /// Average usage
    pub avg_usage: usize,
    /// Total allocated
    pub total_allocated: usize,
    /// Allocation rate (bytes/sec)
    pub allocation_rate: f64,
}

/// Identified bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Type of bottleneck
    pub bottleneck_type: BottleneckType,
    /// Description
    pub description: String,
    /// Severity (0-10)
    pub severity: u8,
    /// Suggested fix
    pub suggestion: String,
}

/// Types of bottlenecks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU-bound computation
    CpuBound,
    /// Memory allocation
    MemoryAllocation,
    /// Cache misses
    CacheMiss,
    /// Lock contention
    LockContention,
    /// I/O wait
    IoWait,
    /// Algorithmic complexity
    AlgorithmicComplexity,
}

impl Profiler {
    /// Create new profiler
    pub fn new() -> Self {
        Self {
            timings: HashMap::new(),
            memory_snapshots: Vec::new(),
            call_graph: CallGraph::new(),
            start_time: Instant::now(),
        }
    }
    
    /// Profile a function execution
    pub fn profile<F, R>(&mut self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let memory_start = self.current_memory_usage();
        
        // Update call graph
        self.call_graph.enter_function(name);
        
        // Execute function
        let result = f();
        
        // Record timing
        let duration = start.elapsed();
        let memory_end = self.current_memory_usage();
        
        self.timings
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(TimingEntry {
                start,
                duration,
                memory_start,
                memory_end,
            });
        
        // Update call graph
        self.call_graph.exit_function(name, duration);
        
        // Take memory snapshot if significant change
        if (memory_end as i64 - memory_start as i64).abs() > 1_000_000 {
            self.memory_snapshots.push(MemorySnapshot {
                timestamp: self.start_time.elapsed(),
                allocated: memory_end,
                in_use: memory_end,
                peak: memory_end.max(memory_start),
            });
        }
        
        result
    }
    
    /// Get current memory usage (simplified)
    fn current_memory_usage(&self) -> usize {
        // In practice, would use system-specific memory queries
        // For now, return a placeholder
        std::mem::size_of::<Self>() * 1000
    }
    
    /// Generate profiling report
    pub fn generate_report(&self) -> ProfileReport {
        let total_runtime = self.start_time.elapsed();
        let functions = self.analyze_functions(&total_runtime);
        let hotspots = self.identify_hotspots(&functions);
        let memory_stats = self.analyze_memory();
        let bottlenecks = self.identify_bottlenecks(&functions, &memory_stats);
        
        ProfileReport {
            total_runtime,
            functions,
            hotspots,
            memory_stats,
            bottlenecks,
        }
    }
    
    /// Analyze function statistics
    fn analyze_functions(&self, total_runtime: &Duration) -> Vec<FunctionStats> {
        self.timings
            .iter()
            .map(|(name, entries)| {
                let total_time: Duration = entries.iter().map(|e| e.duration).sum();
                let avg_time = total_time / entries.len() as u32;
                let min_time = entries.iter().map(|e| e.duration).min().unwrap_or_default();
                let max_time = entries.iter().map(|e| e.duration).max().unwrap_or_default();
                
                FunctionStats {
                    name: name.clone(),
                    total_time,
                    avg_time,
                    min_time,
                    max_time,
                    call_count: entries.len() as u64,
                    time_percentage: (total_time.as_secs_f64() / total_runtime.as_secs_f64()) * 100.0,
                }
            })
            .collect()
    }
    
    /// Identify performance hotspots
    fn identify_hotspots(&self, functions: &[FunctionStats]) -> Vec<Hotspot> {
        let mut hotspots: Vec<_> = functions
            .iter()
            .filter(|f| f.time_percentage > 5.0) // More than 5% of total time
            .map(|f| Hotspot {
                function: f.name.clone(),
                time: f.total_time,
                percentage: f.time_percentage,
                location: None,
            })
            .collect();
        
        // Sort by time percentage
        hotspots.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());
        hotspots.truncate(10); // Top 10 hotspots
        
        hotspots
    }
    
    /// Analyze memory usage
    fn analyze_memory(&self) -> MemoryStats {
        if self.memory_snapshots.is_empty() {
            return MemoryStats {
                peak_usage: 0,
                avg_usage: 0,
                total_allocated: 0,
                allocation_rate: 0.0,
            };
        }
        
        let peak_usage = self.memory_snapshots.iter().map(|s| s.peak).max().unwrap_or(0);
        let avg_usage = self.memory_snapshots.iter().map(|s| s.in_use).sum::<usize>() 
            / self.memory_snapshots.len();
        let total_allocated = self.memory_snapshots.last().map(|s| s.allocated).unwrap_or(0);
        
        let runtime_secs = self.start_time.elapsed().as_secs_f64();
        let allocation_rate = if runtime_secs > 0.0 {
            total_allocated as f64 / runtime_secs
        } else {
            0.0
        };
        
        MemoryStats {
            peak_usage,
            avg_usage,
            total_allocated,
            allocation_rate,
        }
    }
    
    /// Identify performance bottlenecks
    fn identify_bottlenecks(&self, functions: &[FunctionStats], memory: &MemoryStats) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();
        
        // Check for CPU-bound functions
        for func in functions {
            if func.time_percentage > 30.0 {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::CpuBound,
                    description: format!("{} consumes {:.1}% of runtime", func.name, func.time_percentage),
                    severity: ((func.time_percentage / 10.0) as u8).min(10),
                    suggestion: "Consider optimizing algorithm or parallelizing".to_string(),
                });
            }
        }
        
        // Check for memory issues
        if memory.allocation_rate > 100_000_000.0 { // 100MB/s
            bottlenecks.push(Bottleneck {
                bottleneck_type: BottleneckType::MemoryAllocation,
                description: format!("High allocation rate: {:.1} MB/s", memory.allocation_rate / 1_000_000.0),
                severity: 7,
                suggestion: "Use object pools or reduce allocations".to_string(),
            });
        }
        
        // Check for algorithmic complexity
        for func in functions {
            if func.call_count > 10000 && func.avg_time > Duration::from_micros(100) {
                bottlenecks.push(Bottleneck {
                    bottleneck_type: BottleneckType::AlgorithmicComplexity,
                    description: format!("{} called {} times with avg time {:?}", 
                        func.name, func.call_count, func.avg_time),
                    severity: 6,
                    suggestion: "Consider caching results or optimizing algorithm".to_string(),
                });
            }
        }
        
        bottlenecks
    }
}

impl CallGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            call_stack: Vec::new(),
        }
    }
    
    fn enter_function(&mut self, name: &str) {
        self.call_stack.push(name.to_string());
        
        let node = self.nodes.entry(name.to_string()).or_insert(CallNode {
            name: name.to_string(),
            total_time: Duration::default(),
            self_time: Duration::default(),
            call_count: 0,
            children: HashMap::new(),
        });
        
        node.call_count += 1;
        
        // Update parent-child relationship
        if self.call_stack.len() > 1 {
            let parent = &self.call_stack[self.call_stack.len() - 2];
            if let Some(parent_node) = self.nodes.get_mut(parent) {
                *parent_node.children.entry(name.to_string()).or_insert(0) += 1;
            }
        }
    }
    
    fn exit_function(&mut self, name: &str, duration: Duration) {
        self.call_stack.pop();
        
        if let Some(node) = self.nodes.get_mut(name) {
            node.total_time += duration;
            node.self_time += duration; // Will be adjusted for children
        }
    }
}

/// Thread-safe profiler wrapper
pub struct ThreadSafeProfiler {
    inner: Arc<Mutex<Profiler>>,
}

impl ThreadSafeProfiler {
    /// Create new thread-safe profiler
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Profiler::new())),
        }
    }
    
    /// Profile a function
    pub fn profile<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        // For thread safety, we can't hold the lock during function execution
        // So we just time it externally
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        
        // Record the timing
        if let Ok(mut profiler) = self.inner.lock() {
            profiler.timings
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(TimingEntry {
                    start,
                    duration,
                    memory_start: 0,
                    memory_end: 0,
                });
        }
        
        result
    }
    
    /// Generate report
    pub fn generate_report(&self) -> ProfileReport {
        self.inner.lock().unwrap().generate_report()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_profiler() {
        let mut profiler = Profiler::new();
        
        // Profile some functions
        profiler.profile("test_function", || {
            thread::sleep(Duration::from_millis(10));
            42
        });
        
        profiler.profile("test_function", || {
            thread::sleep(Duration::from_millis(5));
            24
        });
        
        let report = profiler.generate_report();
        assert!(!report.functions.is_empty());
        assert_eq!(report.functions[0].call_count, 2);
    }
    
    #[test]
    fn test_thread_safe_profiler() {
        let profiler = ThreadSafeProfiler::new();
        
        let p1 = profiler.inner.clone();
        let handle = thread::spawn(move || {
            let mut p = p1.lock().unwrap();
            p.profile("thread_function", || {
                thread::sleep(Duration::from_millis(1));
            });
        });
        
        handle.join().unwrap();
        
        let report = profiler.generate_report();
        assert!(!report.functions.is_empty());
    }
}