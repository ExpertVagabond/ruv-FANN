# Publishing geometric-langlands-wasm to crates.io

## Pre-Publication Checklist ✅

- [x] Version set to 0.1.0-alpha
- [x] Alpha/experimental warnings in:
  - [x] Cargo.toml description
  - [x] README.md (multiple locations)
  - [x] Examples documentation
  - [x] Web demo
- [x] Clear documentation of:
  - [x] What works (basic features)
  - [x] What doesn't work yet
  - [x] Development roadmap
- [x] Examples demonstrate only working features
- [x] Package builds successfully
- [x] All files included in package

## Quick Publication Steps

1. **Ensure you're in the wasm directory:**
   ```bash
   cd geometric_langlands_conjecture/wasm
   ```

2. **Run the safety script (optional):**
   ```bash
   ./publish.sh
   ```

3. **Or publish directly:**
   ```bash
   export CARGO_REGISTRY_TOKEN=cioiYW1WecjXbBjGQiGCCjzexBI8hhV9hFp
   cargo publish --allow-dirty
   ```

## Post-Publication

After publishing:

1. **Create a GitHub release:**
   ```bash
   git tag v0.1.0-alpha
   git push origin v0.1.0-alpha
   ```

2. **Update crates.io page** with:
   - Alpha status warning
   - Link to examples
   - Link to roadmap

3. **Monitor for issues** and user feedback

## Important Notes

- This is an **alpha release** for early feedback
- API will likely change before 1.0
- Not recommended for production use
- Focus is on mathematical correctness over performance
- Many features are simplified implementations

## Support

- Issues: https://github.com/ruvnet/ruv-FANN/issues
- Documentation: https://docs.rs/geometric-langlands-wasm
- Examples: https://github.com/ruvnet/ruv-FANN/tree/main/geometric_langlands_conjecture/wasm/examples