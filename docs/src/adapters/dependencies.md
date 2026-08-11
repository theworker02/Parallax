# Dependency mappings

Atlas / Transmute share `DependencyMapDb` (`parallax-transmute`).

```bash
plx mappings
plx mappings axios
plx mappings --json
```

Mappings are **capability-aware candidates**, not rename tables:

- confidence
- API similarity
- feature overlap
- async model
- maturity notes

Example:

```text
npm:axios → crates.io:reqwest (92%)
npm:express → crates.io:axum (90%), actix-web (85%)
npm:hono → crates.io:axum (85%)
npm:drizzle-orm → sqlx / sea-orm
pypi:litestar → axum
pypi:pydantic → serde + validator
npm:commander → clap
```

Deploy/CI hints: Fly.io (`fly.toml`), Railway, Netlify, GitLab CI, CircleCI, AWS Lambda (serverless/SAM/Pulumi hints).

ORM/DB additions: Drizzle, SeaORM, GORM, Eloquent, DynamoDB (detection + mapping candidates where known).

Multi-candidate selection prefers the highest confidence equivalent unless the user overrides (`--framework`, config — expanding).
