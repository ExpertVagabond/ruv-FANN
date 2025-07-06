#!/bin/bash

# Script to fix JavaScript linting issues in CI
echo "Starting JavaScript linting fixes..."

# First check in node_modules after npm installation
if [ -d "node_modules/ruv-swarm" ]; then
  echo "Found ruv-swarm in node_modules, fixing linting issues there..."
  
  # Fix missing radix parameter in cli-diagnostics.js
  find node_modules/ruv-swarm -name "cli-diagnostics.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/parseInt(\([^,)]*\))/parseInt(\1, 10)/g' "$file"
  done
  
  # Fix missing braces after if conditions in diagnostics.js
  find node_modules/ruv-swarm -name "diagnostics.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' "$file"
  done
  
  # Fix missing braces after if conditions in logger.js
  find node_modules/ruv-swarm -name "logger.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' "$file"
  done
  
  # Fix 'let' to 'const' for variables never reassigned in mcp-server-reliability-test.js
  find node_modules/ruv-swarm -name "mcp-server-reliability-test.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/let initializationLogs/const initializationLogs/g' "$file"
  done
  
  echo "JavaScript linting fixes in node_modules completed."
fi

# Also check in ruv-swarm/npm directory if it exists
if [ -d "ruv-swarm/npm" ]; then
  echo "Checking ruv-swarm/npm directory..."
  
  # Fix missing radix parameter in cli-diagnostics.js
  find ruv-swarm/npm -name "cli-diagnostics.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/parseInt(\([^,)]*\))/parseInt(\1, 10)/g' "$file"
  done
  
  # Fix missing braces after if conditions in diagnostics.js
  find ruv-swarm/npm -name "diagnostics.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' "$file"
  done
  
  # Fix missing braces after if conditions in logger.js
  find ruv-swarm/npm -name "logger.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' "$file"
  done
  
  # Fix 'let' to 'const' for variables never reassigned in mcp-server-reliability-test.js
  find ruv-swarm/npm -name "mcp-server-reliability-test.js" -type f | while read file; do
    echo "Fixing $file"
    sed -i 's/let initializationLogs/const initializationLogs/g' "$file"
  done
  
  echo "JavaScript linting fixes in ruv-swarm/npm completed."
fi

echo "All JavaScript linting fixes completed."
