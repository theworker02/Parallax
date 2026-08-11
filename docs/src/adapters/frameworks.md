# Framework adapters

Frameworks are first-class Atlas adapters (`AdapterKind::Framework`).

## Built-ins (detection)

| Adapter | Maturity | Notes |
|---------|----------|-------|
| Express | stable | Pack: → Axum |
| FastAPI | stable | Pack path → Axum |
| Axum | stable | Target-side |
| NestJS, Fastify, Flask, Django, Gin, Chi, Hono, Koa, Fiber, Echo, Rocket, Litestar | beta | Detection + mapping hints |
| Spring Boot, ASP.NET, Rails, Laravel, Next.js, Ktor, Vapor, Sanic, Phoenix, Sinatra | experimental | Detect only |
| Quarkus, Micronaut, Symfony, Slim, Beego, Buffalo | experimental | JVM/PHP/Go detection |

## Web frontends (`AdapterKind::WebFrontend`)

Compose with backend frameworks (e.g. Express + React). Detection only today:

| Adapter | Maturity |
|---------|----------|
| React, Vue, Svelte, Solid, Angular | experimental |

## Preferred mappings

```text
Express  → Rust: Axum | Go: Chi | Python: FastAPI
FastAPI  → Rust: Axum | Go: Chi | TypeScript: Fastify
```

Scores come from dependency knowledge (`plx mappings`) and stack suggestion (`plx analyze --to` / `plx explain-stack`).

## Contract (intent)

Framework adapters should eventually:

- detect presence
- extract routes / middleware / services
- map to a target framework via a `MigrationPack`

Today, Express→Axum remains the implemented Transmute pack; other frameworks contribute detection and planning honesty.
