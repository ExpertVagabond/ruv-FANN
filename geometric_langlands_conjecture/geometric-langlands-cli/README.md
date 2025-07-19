# Geometric Langlands CLI

A user-friendly command-line interface for the Geometric Langlands computational framework, making advanced mathematical computations accessible to researchers and mathematicians.

## Features

- **Interactive REPL** for mathematical computations
- **Batch processing** of correspondences and verifications
- **Rich visualizations** of mathematical objects
- **Persistent storage** with SQLite database
- **Multiple export formats** (JSON, LaTeX, Mathematica, SageMath, Python)
- **Configuration management** with TOML files
- **Progress tracking** with detailed progress bars

## Installation

```bash
# Install from source
git clone https://github.com/ruvnet/ruv-FANN.git
cd ruv-FANN/geometric_langlands_conjecture/geometric-langlands-cli
cargo install --path .
```

## Quick Start

### Start Interactive REPL
```bash
langlands repl
```

### Compute Correspondences
```bash
# Verify Langlands correspondence
langlands compute correspondence --input "GL(3)" --output results.json

# Compute Hecke eigenvalues
langlands compute hecke --input "GL(2)" --parallel

# Evaluate L-functions
langlands compute l-function --input "eisenstein" --output l_values.csv
```

### Visualizations
```bash
# Visualize Hecke eigenvalues
langlands visual hecke-eigenvalues --interactive --resolution 1920x1080

# Plot L-function
langlands visual l-function --output l_function.png

# Visualize moduli space
langlands visual moduli-space --interactive
```

### Train Neural Networks
```bash
# Train on correspondence patterns
langlands train --dataset training_data.json --epochs 100 --save-model model.bin

# Use custom architecture
langlands train --dataset data.json --architecture deep --learning-rate 0.001
```

### Verify Mathematical Properties
```bash
# Verify Ramanujan conjecture
langlands verify ramanujan --input "GL(2)" --depth deep --proof

# Check functoriality
langlands verify functoriality --input "GL(2)->GL(3)" --proof

# Verify reciprocity laws
langlands verify reciprocity --depth exhaustive
```

### Export Results
```bash
# Export to LaTeX
langlands export recent --format latex --output paper.tex --metadata

# Export to Mathematica
langlands export computation_id --format mathematica --output notebook.nb

# Export to Python
langlands export all --format python --output analysis.py
```

## Commands

### Core Commands

- `langlands compute <type>` - Run mathematical computations
- `langlands visual <type>` - Create visualizations  
- `langlands train` - Train neural networks
- `langlands verify <property>` - Verify mathematical properties
- `langlands export <source>` - Export results in various formats
- `langlands repl` - Start interactive session

### Computation Types

- `correspondence` - Langlands correspondence verification
- `hecke` - Hecke operator eigenvalues
- `l-function` - L-function evaluations
- `trace-formula` - Trace formula computations
- `spectral` - Spectral decomposition
- `functoriality` - Functorial lifts
- `ramanujan` - Ramanujan conjecture verification

### Visualization Types

- `sheaf` - Perverse sheaf structure
- `representation` - Galois representations
- `moduli-space` - Moduli space of bundles
- `spectral-curve` - Spectral curves
- `hecke-eigenvalues` - Hecke eigenvalue plots
- `l-function` - L-function plots
- `correspondence` - Langlands correspondence diagram

### Verification Properties

- `correspondence` - Langlands correspondence
- `functoriality` - Functorial properties
- `reciprocity` - Reciprocity laws
- `ramanujan` - Ramanujan conjecture
- `selberg` - Selberg trace formula
- `riemann-hypothesis` - Generalized Riemann hypothesis
- `local-global` - Local-global principle

### Database Management

```bash
# Initialize database
langlands db init

# List stored computations
langlands db list --limit 20

# Show computation details
langlands db show <computation_id>

# Export/import database
langlands db export database_backup.json
langlands db import database_backup.json
```

### Configuration

```bash
# Show current configuration
langlands config show

# Set configuration values
langlands config set neural.learning_rate 0.001
langlands config set visualization.default_resolution 1920x1080

# Reset to defaults
langlands config reset
```

## Interactive REPL

The REPL provides an interactive environment for mathematical exploration:

```
langlands> create group g GL 3
Created group g: GL(3)

langlands> create form f g 2
Created automorphic form f: Eisenstein series of weight 2

langlands> compute correspondence
Langlands correspondence: computed ✓
Verified: ✓

langlands> compute hecke 5
T_5(f) = 2.236068

langlands> plot hecke
Plot opened in viewer

langlands> verify ramanujan
Ramanujan conjecture at p=2: ✓

langlands> save session.json
Session saved to: session.json
```

### REPL Commands

- `help` - Show help message
- `vars` - List all variables
- `create <type> <name> [args]` - Create mathematical objects
- `compute <operation>` - Perform computations
- `plot <type>` - Generate plots
- `verify <property>` - Verify properties
- `save/load <file>` - Session management

## Configuration

Configuration is stored in `~/.config/langlands-cli/config.toml`:

```toml
default_precision = 64
max_iterations = 10000
convergence_threshold = 1e-10

[computation]
enable_parallel = true
enable_gpu = false
cache_results = true

[visualization]
default_resolution = [800, 600]
color_scheme = "viridis"
enable_latex = true

[neural]
default_architecture = "langlands_v1"
learning_rate = 0.001
batch_size = 32

[repl]
history_size = 1000
auto_save = true
prompt = "langlands> "
```

## Output Formats

### JSON
```json
{
  "computation": {
    "type": "correspondence",
    "verified": true,
    "details": "..."
  },
  "metadata": {
    "version": "0.1.0",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### LaTeX
```latex
\documentclass{article}
\begin{document}
\section{Langlands Correspondence}
The correspondence between automorphic forms and Galois representations...
\begin{align}
L(s) &= 1.234567
\end{align}
\end{document}
```

### Mathematica
```mathematica
correspondence = {
  "type" -> "Langlands",
  "verified" -> True,
  "lFunction" -> LFunction[s]
};
```

## Examples

### Research Workflow

```bash
# 1. Set up computation
langlands config set computation.enable_parallel true
langlands config set neural.batch_size 64

# 2. Verify correspondences for GL(n)
for n in 2 3 4; do
  langlands compute correspondence --input "GL($n)" --output "gl${n}_results.json"
done

# 3. Train neural network on patterns
langlands train --dataset combined_data.json --architecture deep --epochs 200

# 4. Generate visualizations
langlands visual correspondence --output correspondence_diagram.svg
langlands visual hecke-eigenvalues --resolution 2560x1440 --output eigenvalues.png

# 5. Export for publication
langlands export recent --format latex --metadata --output paper.tex
```

### Batch Verification

```bash
#!/bin/bash
# Batch verify multiple properties

properties=("correspondence" "functoriality" "ramanujan" "reciprocity")

for prop in "${properties[@]}"; do
  echo "Verifying $prop..."
  langlands verify "$prop" --depth standard --proof > "${prop}_verification.txt"
done

# Generate summary report
langlands export all --format json --output verification_summary.json
```

## Performance

The CLI is optimized for mathematical computations:

- **Parallel processing** with configurable thread count
- **GPU acceleration** for supported operations (CUDA)
- **Intelligent caching** to avoid redundant computations
- **Memory-efficient** algorithms for large-scale problems
- **Progress tracking** for long-running computations

## Contributing

We welcome contributions! Please see the main repository's contributing guidelines.

## License

MIT License - see LICENSE file for details.

## Citation

If you use this tool in your research, please cite:

```bibtex
@software{geometric_langlands_cli,
  title = {Geometric Langlands CLI},
  author = {ruv-FANN Contributors},
  url = {https://github.com/ruvnet/ruv-FANN},
  version = {0.1.0},
  year = {2024}
}
```