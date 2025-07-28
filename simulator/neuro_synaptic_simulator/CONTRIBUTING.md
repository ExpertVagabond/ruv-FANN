# Contributing to Neuro-Synaptic Simulator

Thank you for your interest in contributing to the Neuro-Synaptic Chip Simulator! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community](#community)

## Code of Conduct

This project adheres to a Code of Conduct that all contributors are expected to follow. Please be respectful, inclusive, and professional in all interactions.

- Be welcoming to newcomers and encourage diverse contributions
- Be respectful of differing viewpoints and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

1. **Fork the Repository**: Click the "Fork" button on the GitHub repository page
2. **Clone Your Fork**: 
   ```bash
   git clone https://github.com/your-username/ruv-FANN.git
   cd ruv-FANN/simulator/neuro_synaptic_simulator
   ```
3. **Add Upstream Remote**:
   ```bash
   git remote add upstream https://github.com/ruvnet/ruv-FANN.git
   ```

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Cargo and rustup
- Git
- C++ compiler (for Wasmtime dependencies)
- 8GB RAM minimum
- Optional: Docker for isolated development

### Initial Setup

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch cargo-criterion cargo-flamegraph

# Install pre-commit hooks (optional but recommended)
pip install pre-commit
pre-commit install

# Build the project
cargo build

# Run tests to verify setup
cargo test
```

### IDE Setup

We recommend using VS Code or IntelliJ IDEA with Rust plugins:

**VS Code Extensions**:
- rust-analyzer
- CodeLLDB (for debugging)
- Better TOML
- crates

**IntelliJ IDEA**:
- Rust plugin
- TOML plugin

## How to Contribute

### Types of Contributions

1. **Bug Fixes**: Fix issues reported in GitHub Issues
2. **Feature Implementation**: Implement features from the roadmap
3. **Performance Improvements**: Optimize existing code
4. **Documentation**: Improve or add documentation
5. **Tests**: Add test coverage
6. **Examples**: Create example programs

### Finding Work

1. Check [GitHub Issues](https://github.com/ruvnet/ruv-FANN/issues) for open tasks
2. Look for issues labeled `good first issue` for beginners
3. Review the [FEATURES.md](FEATURES.md) file for planned features
4. Propose new features through GitHub Issues

### Creating an Issue

Before starting work, create or find an issue:

```markdown
**Title**: Clear, concise description

**Description**: 
- What is the current behavior?
- What is the expected behavior?
- Steps to reproduce (if applicable)

**Additional Context**:
- Environment details
- Related issues or PRs
- Proposed solution (if any)
```

## Coding Standards

### Rust Style Guide

We follow the official Rust style guide with some additions:

```rust
// Good: Clear, documented function
/// Calculates the memory offset for a given core ID.
/// 
/// # Arguments
/// * `core_id` - The ID of the processing core (0-255)
/// 
/// # Returns
/// The byte offset in shared memory
pub fn calculate_core_offset(core_id: u8) -> usize {
    const CORE_MEMORY_SIZE: usize = 32 * 1024; // 32KB per core
    core_id as usize * CORE_MEMORY_SIZE
}

// Bad: Unclear, undocumented
pub fn calc_off(id: u8) -> usize {
    id as usize * 32768
}
```

### Best Practices

1. **Error Handling**: Use `Result<T, E>` and `?` operator
   ```rust
   fn load_module(path: &Path) -> Result<Module, SimulatorError> {
       let bytes = std::fs::read(path)?;
       Module::new(&engine, &bytes).map_err(SimulatorError::WasmError)
   }
   ```

2. **Memory Safety**: Avoid `unsafe` unless absolutely necessary
   ```rust
   // Prefer safe abstractions
   let memory_slice = &memory[offset..offset + size];
   
   // Document unsafe blocks when needed
   unsafe {
       // SAFETY: offset and size are bounds-checked above
       std::ptr::copy_nonoverlapping(src, dst, size);
   }
   ```

3. **Concurrency**: Use appropriate synchronization
   ```rust
   use std::sync::{Arc, Mutex};
   use parking_lot::RwLock; // Prefer parking_lot for performance
   ```

4. **Performance**: Profile before optimizing
   ```rust
   #[cfg(feature = "profile")]
   let _timer = Timer::new("critical_section");
   ```

### Code Formatting

Always run before committing:

```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings

# Fix common issues
cargo fix --allow-dirty
```

## Testing Guidelines

### Test Organization

```
tests/
├── unit/           # Unit tests for individual components
│   ├── core/       # Core functionality tests
│   ├── memory/     # Memory subsystem tests
│   └── wasm/       # WASM integration tests
├── integration/    # Full system integration tests
└── benchmarks/     # Performance benchmarks
```

### Writing Tests

1. **Unit Tests**: Test individual functions and modules
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_memory_allocation() {
           let pool = MemoryPool::new(28 * 1024 * 1024);
           assert_eq!(pool.available(), 28 * 1024 * 1024);
       }

       #[test]
       #[should_panic(expected = "out of memory")]
       fn test_allocation_failure() {
           let pool = MemoryPool::new(1024);
           pool.allocate(2048); // Should panic
       }
   }
   ```

2. **Integration Tests**: Test complete workflows
   ```rust
   // tests/integration/simulation.rs
   #[test]
   fn test_full_simulation() {
       let simulator = Simulator::new(Config::default());
       let result = simulator.run("models/test.wasm", 1000);
       assert!(result.is_ok());
       assert_eq!(result.unwrap().cores_used, 256);
   }
   ```

3. **Benchmarks**: Measure performance
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn benchmark_inference(c: &mut Criterion) {
       c.bench_function("inference_256_cores", |b| {
           b.iter(|| {
               simulate_inference(black_box(256))
           })
       });
   }
   ```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_memory_allocation

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

## Documentation

### Code Documentation

Document all public APIs:

```rust
/// Represents a single processing unit in the neuro-synaptic chip.
/// 
/// Each processing unit can execute WASM code independently while
/// sharing access to the global memory pool.
/// 
/// # Example
/// ```
/// let mut core = ProcessingUnit::new(0);
/// core.load_task(task);
/// let result = core.execute()?;
/// ```
pub struct ProcessingUnit {
    /// Unique identifier for this core (0-255)
    pub id: u8,
    // Private fields...
}
```

### Documentation Standards

1. Document all public items
2. Include examples where helpful
3. Explain complex algorithms
4. Keep documentation up-to-date
5. Use proper markdown formatting

### Building Documentation

```bash
# Build and open documentation
cargo doc --open

# Build with private items
cargo doc --document-private-items

# Check documentation examples
cargo test --doc
```

## Pull Request Process

### Before Submitting

1. **Update your fork**:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**:
   - Write clean, documented code
   - Add tests for new functionality
   - Update documentation
   - Ensure all tests pass

4. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add new feature

   - Detailed description of changes
   - Fixes #123"
   ```

   Follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation changes
   - `test:` Test additions/changes
   - `perf:` Performance improvements
   - `refactor:` Code refactoring

### Submitting a PR

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**:
   - Go to GitHub and click "New Pull Request"
   - Select your branch
   - Fill out the PR template
   - Link related issues

3. **PR Description Template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Performance improvement
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Comments added for complex code
   - [ ] Documentation updated
   - [ ] No new warnings

   Fixes #(issue number)
   ```

### Review Process

1. **Automated Checks**: CI will run tests and lints
2. **Code Review**: Maintainers will review your code
3. **Address Feedback**: Make requested changes
4. **Merge**: Once approved, your PR will be merged

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: General questions and ideas
- **Discord**: Real-time chat (if available)
- **Email**: For private concerns

### Getting Help

If you need help:

1. Check existing documentation
2. Search closed issues
3. Ask in GitHub Discussions
4. Join our Discord server

### Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project documentation

## Advanced Contributing

### Performance Optimization

When optimizing performance:

1. **Profile First**: Use flamegraph or perf
2. **Benchmark**: Create reproducible benchmarks
3. **Document**: Explain optimizations in comments
4. **Test**: Ensure correctness isn't compromised

### Adding Dependencies

Before adding new dependencies:

1. Check if functionality exists in std
2. Evaluate maintenance status
3. Consider compile time impact
4. Document why it's needed
5. Prefer well-known crates

### Architecture Changes

For significant architectural changes:

1. Create an RFC issue first
2. Discuss with maintainers
3. Update architecture documentation
4. Consider backward compatibility
5. Plan migration path

## Thank You!

Your contributions make this project better for everyone. Whether it's fixing a typo, adding a test, or implementing a major feature, every contribution is valued and appreciated.

Happy coding! 🚀