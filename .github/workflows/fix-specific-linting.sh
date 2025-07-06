#!/bin/bash

echo "Fixing specific linting issues in ruv-swarm npm package..."

# Check if ruv-swarm/npm directory exists
if [ ! -d "ruv-swarm/npm" ]; then
  echo "Error: ruv-swarm/npm directory not found"
  exit 1
fi

# Fix 'let' to 'const' in test file
if [ -f "ruv-swarm/npm/test/mcp-server-reliability-test.js" ]; then
  echo "Fixing 'let' to 'const' in mcp-server-reliability-test.js"
  sed -i 's/let initializationLogs/const initializationLogs/g' ruv-swarm/npm/test/mcp-server-reliability-test.js
fi

# Fix missing braces after 'if' conditions in logger.js
if [ -f "ruv-swarm/npm/src/logger.js" ]; then
  echo "Fixing missing braces in logger.js"
  # Line 277
  sed -i '277s/if (.*)/&{/g' ruv-swarm/npm/src/logger.js
  sed -i '277a\}' ruv-swarm/npm/src/logger.js
  
  # Line 191
  sed -i '191s/if (.*)/&{/g' ruv-swarm/npm/src/logger.js
  sed -i '191a\}' ruv-swarm/npm/src/logger.js
  
  # Line 155
  sed -i '155s/if (.*)/&{/g' ruv-swarm/npm/src/logger.js
  sed -i '155a\}' ruv-swarm/npm/src/logger.js
  
  # Line 103
  sed -i '103s/if (.*)/&{/g' ruv-swarm/npm/src/logger.js
  sed -i '103a\}' ruv-swarm/npm/src/logger.js
  
  # Line 95
  sed -i '95s/if (.*)/&{/g' ruv-swarm/npm/src/logger.js
  sed -i '95a\}' ruv-swarm/npm/src/logger.js
fi

# Fix missing braces in diagnostics.js
if [ -f "ruv-swarm/npm/src/diagnostics.js" ]; then
  echo "Fixing missing braces in diagnostics.js"
  # Line 219
  sed -i '219s/if (.*)/&{/g' ruv-swarm/npm/src/diagnostics.js
  sed -i '219a\}' ruv-swarm/npm/src/diagnostics.js
fi

# Fix missing radix parameter in cli-diagnostics.js
if [ -f "ruv-swarm/npm/src/cli-diagnostics.js" ]; then
  echo "Fixing missing radix parameter in cli-diagnostics.js"
  # Lines 117-118
  sed -i 's/parseInt(\([^,)]*\))/parseInt(\1, 10)/g' ruv-swarm/npm/src/cli-diagnostics.js
fi

echo "Linting fixes completed."
