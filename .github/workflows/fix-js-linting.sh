#!/bin/bash

# Script to fix JavaScript linting issues in CI
echo "Starting JavaScript linting fixes..."

# Navigate to the ruv-swarm/npm directory if it exists
if [ -d "ruv-swarm/npm" ]; then
  cd ruv-swarm/npm
  
  # Fix missing radix parameter in cli-diagnostics.js
  if [ -f "src/cli-diagnostics.js" ]; then
    echo "Fixing src/cli-diagnostics.js"
    sed -i 's/parseInt(\([^,)]*\))/parseInt(\1, 10)/g' src/cli-diagnostics.js
  fi
  
  # Fix missing braces after if conditions in diagnostics.js
  if [ -f "src/diagnostics.js" ]; then
    echo "Fixing src/diagnostics.js"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' src/diagnostics.js
  fi
  
  # Fix missing braces after if conditions in logger.js
  if [ -f "src/logger.js" ]; then
    echo "Fixing src/logger.js"
    sed -i 's/\(if ([^{]*)\) \([^{]\)/\1 {\n  \2\n}/g' src/logger.js
  fi
  
  # Fix 'let' to 'const' for variables never reassigned in mcp-server-reliability-test.js
  if [ -f "test/mcp-server-reliability-test.js" ]; then
    echo "Fixing test/mcp-server-reliability-test.js"
    sed -i 's/let initializationLogs/const initializationLogs/g' test/mcp-server-reliability-test.js
  fi
  
  echo "JavaScript linting fixes completed."
else
  echo "ruv-swarm/npm directory not found. Skipping JavaScript linting fixes."
fi
