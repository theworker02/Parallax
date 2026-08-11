# Publishing adapters

**Today:** adapters ship built-in with Parallax.

**Planned distribution:**

```text
.parallax/adapters/          project-local
~/.parallax/adapters/        user-installed
package registries           versioned adapter crates / packs
```

## Lockfiles

`parallax.lock` pins adapter versions for reproducible migrations.

```bash
plx analyze . --write-lock
plx adapters update --check
```

Breaking mapping changes between adapter versions must surface as review-required diagnostics (not silent semantic drift).

## Checklist before publishing

1. Valid `adapter.toml` / manifest
2. Capability flags complete
3. Bronze fixtures green
4. Permissions minimized
5. Determinism verified
6. Document maturity honestly
