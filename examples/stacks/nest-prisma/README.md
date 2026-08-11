# NestJS + Prisma stack fixture

Minimal manifest for Atlas stack detection demos — not a runnable app.

```bash
plx analyze examples/stacks/nest-prisma --to rust
plx adapters | findstr /i nest
```

Expected detections include NestJS, Prisma, TypeScript, npm, Vitest, Prettier, and ESLint.
