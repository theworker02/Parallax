# Source-language adapters

Source adapters normalize into shared semantic structures (functions, types, modules, control flow, async, …) without leaking language-specific ASTs into the planner.

| Language | Maturity | Notes |
|----------|----------|-------|
| TypeScript / JavaScript | stable | Transmute frontend via TS compiler API |
| Python | beta | Expanding |
| Go, Java, Kotlin, C#, Ruby, PHP | experimental | Detection + connectors |
| Swift, Dart, Lua | scaffold | Identity / detect |
| C / C++ | parse_only | No claim of migration |

See also [Language connectors](connectors.md).
