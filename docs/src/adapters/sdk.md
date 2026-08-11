# Adapter SDK

Crate: `parallax-adapter-sdk`

## Base trait

```rust
trait ParallaxAdapter {
    fn manifest(&self) -> AdapterManifest;
    fn detect(&self, context: &ProjectContext) -> DetectionResult;
    fn capabilities(&self) -> AdapterCapabilities;
}
```

Specialized markers: `SourceLanguageAdapter`, `TargetLanguageAdapter`, `FrameworkAdapter`, `DependencyAdapter`, `BuildSystemAdapter`, `TestFrameworkAdapter`, `DatabaseAdapter`, `ConfigurationAdapter`, `DeploymentAdapter`, `VerificationAdapter`.

## Manifest

Every adapter exposes `AdapterManifest`:

- `id` — stable (`parallax.typescript.source`)
- `version` — independently versioned with the product today; package distribution later
- `adapter_type` — source-language, framework, orm, …
- `languages` / `ecosystems`
- `maturity` / `conformance` (Bronze / Silver / Gold)
- `priority` — conflict resolution
- `owns` — semantic nodes this adapter transforms
- `permissions` — capability sandbox for third-party adapters
- `sdk_version` — `ADAPTER_SDK_VERSION`

## Capabilities

Machine-readable flags (`FULL` / `PARTIAL` / `UNSUPPORTED`), e.g. TypeScript source:

```text
parsing...................FULL
types.....................FULL
decorators................PARTIAL
dynamic_eval..............UNSUPPORTED
```

## Detection

`ProjectContext` carries root, relative files, manifests, package names, language mix, and CLI hints (`to`).

`DetectionResult` includes confidence, evidence, and optional `owns_nodes`.

## Developing an adapter

1. Read [examples/custom-adapter](../../../examples/custom-adapter/README.md)
2. Implement `ParallaxAdapter` (+ specialized trait)
3. Register via `AdapterRegistry::register` or ship under `.parallax/adapters/` (discovery planned)
4. Aim for Bronze → Silver → Gold conformance ([conformance](conformance.md))

Scaffold command (`plx adapter new`) is stubbed; use the example tree as the template today.
