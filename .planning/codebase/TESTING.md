# Testing Patterns

**Analysis Date:** 2026-06-13

## Test Framework

**Runner:**
- **TypeScript:** Vitest (v4.1.8)
- Config: `vitest.config.ts`
- **Rust:** Built-in `cargo test`

**Assertion Library:**
- **TypeScript:** Vitest built-in assertions (`expect`)
- **Rust:** Standard macros (`assert!`, `assert_eq!`)

**Run Commands:**
```bash
npm run test           # Run all TS tests via vitest
npm test               # Alias for TS tests
cargo test             # Run all Rust tests
```

## Test File Organization

**Location:**
- **TypeScript:** Co-located with the source files being tested.
- **Rust:** In-file testing modules (`#[cfg(test)] mod tests { ... }`) usually placed at the bottom of the source file.

**Naming:**
- **TypeScript:** `[filename].test.ts` (e.g., `device.test.ts`, `archive.test.ts`)
- **Rust:** Test modules named `tests`, test functions prefixed with `test_` (e.g., `fn test_verify_checksum_success()`).

**Structure:**
```
src/
  types/
    device.ts
    device.test.ts
src-tauri/
  src/
    download.rs (includes mod tests)
```

## Test Structure

**Suite Organization:**
```typescript
// TypeScript (Vitest)
import { describe, expect, it } from "vitest";

describe("functionName", () => {
	it("should do something", () => {
        // test body
	});
});
```

```rust
// Rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // test body
    }
}
```

**Patterns:**
- Setup and teardown rely on framework standards (like `beforeEach` in TS, though often variables are just initialized per test).
- Assertion pattern follows AAA (Arrange, Act, Assert).

## Mocking

**Framework:** `tempfile` crate for Rust filesystem mocking.

**Patterns:**
```rust
// Rust temporary file creation for filesystem tests
let mut temp_file = tempfile::NamedTempFile::new().unwrap();
write!(temp_file, "test content").unwrap();
// act on temp_file.path()
```

**What to Mock:**
- File system access and OS-level operations (handled via temporary files rather than pure function mocking).
- Network requests in the Rust backend would likely be mocked via interceptors or local servers, though primarily file writing/checksum logic is tested using literal files.

## Fixtures and Factories

**Test Data:**
```typescript
// Usually constructed inline or using constant string properties
const profile = getDeviceProfile("trimui-brick");
expect(profile?.name).toBe("TrimUI Brick");
```

**Location:**
- Kept inline within the test files to reduce cross-file dependency complexity for small unit tests.

## Coverage

**Requirements:** None explicitly enforced in CI configurations, though coverage tooling is installed.

**View Coverage:**
- The `@vitest/coverage-v8` package is installed, allowing for coverage reports to be generated if the `vitest run --coverage` flag is used.

## Test Types

**Unit Tests:**
- Heavily utilized for both TS and Rust.
- TS focuses on data transformation, type correctness, and lookup tables (e.g., parsing device configs).
- Rust tests focus heavily on I/O boundaries, file parsing, and state validation (e.g., checksum verifications, zip extraction).

**Integration Tests:**
- Handled primarily by Tauri's cross-communication layer, though specific integration suites were not deeply observed.

**E2E Tests:**
- Not currently set up in the MVP phase.

## Common Patterns

**Async Testing:**
- Handled natively in both Vitest (via `async/await` in `it`) and Rust (`#[tokio::test]` if async is needed, though unit tests observed were synchronous).

**Error Testing:**
- Typically checks for expected failure cases directly.
```rust
// Rust
let result = verify_checksum(temp_file.path().to_str().unwrap(), "wrong_checksum");
assert!(!result.unwrap());
```

---

*Testing analysis: 2026-06-13*
