# Testing adapters

Test-framework adapters detect runners (Jest, Vitest, Mocha, pytest, unittest, cargo test / Criterion, Go testing, JUnit, Kotest, NUnit, XCTest, Dart test, RSpec, PHPUnit, …).

Build-system adapters also cover pnpm, Yarn, Bun, uv, Poetry, CMake, and Meson (manifest/lockfile detection).

## Assertion IR (direction)

Future emission uses language-independent `AssertionIR` (`Equal`, `Throws`, `Snapshot`, …). Vitest/Jest → `cargo test` is the Tier-1 path used by weather-api.

## Mocking

`jest.mock`, `unittest.mock`, Mockito, etc. often require manual review when no safe equivalent exists — Atlas reports maturity honestly rather than inventing mocks.
