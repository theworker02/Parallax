//! Built-in Atlas adapters (language, framework, build, test, deploy, …).

use crate::registry::{AdapterRegistry, SimpleAdapter};
use parallax_adapter_sdk::{
    AdapterCapabilities, AdapterKind, AdapterManifest, AdapterMaturity, CapabilitySupport,
    DetectionConfidence, DetectionResult, ProjectContext,
};

pub fn register_all(reg: &mut AdapterRegistry) {
    register_languages(reg);
    register_frameworks(reg);
    register_build(reg);
    register_tests(reg);
    register_data(reg);
    register_deploy(reg);
    register_cli(reg);
    register_serialization(reg);
    register_formatters(reg);
    register_linters(reg);
    register_codegen(reg);
    register_desktop_gui(reg);
    register_pairs(reg);
}

fn add(
    reg: &mut AdapterRegistry,
    mut manifest: AdapterManifest,
    caps: AdapterCapabilities,
    detect: fn(&ProjectContext) -> DetectionResult,
) {
    if manifest.priority == 0 {
        manifest.priority = default_priority(manifest.adapter_type);
    }
    reg.register(
        SimpleAdapter {
            manifest,
            capabilities: caps,
            detect_fn: detect,
        }
        .arc(),
    );
}

fn default_priority(kind: AdapterKind) -> i32 {
    match kind {
        AdapterKind::WebFrontend | AdapterKind::Framework => 80,
        AdapterKind::Orm => 70,
        AdapterKind::Database => 60,
        AdapterKind::TestFramework => 50,
        AdapterKind::BuildSystem => 40,
        AdapterKind::Deployment => 30,
        AdapterKind::SourceLanguage | AdapterKind::TargetLanguage => 20,
        AdapterKind::Runtime => 15,
        AdapterKind::Formatter | AdapterKind::Linter => 12,
        AdapterKind::Codegen | AdapterKind::DesktopGui => 11,
        _ => 10,
    }
}

fn register_languages(reg: &mut AdapterRegistry) {
    // Source
    for (id, name, langs, maturity, caps, detect) in [
        (
            "parallax.typescript.source",
            "TypeScript Source Adapter",
            &["typescript", "javascript"][..],
            AdapterMaturity::Stable,
            AdapterCapabilities::typescript_source(),
            detect_typescript as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.javascript.source",
            "JavaScript Source Adapter",
            &["javascript"],
            AdapterMaturity::Stable,
            AdapterCapabilities::typescript_source(),
            detect_javascript,
        ),
        (
            "parallax.python.source",
            "Python Source Adapter",
            &["python"],
            AdapterMaturity::Beta,
            AdapterCapabilities::python_source(),
            detect_python,
        ),
        (
            "parallax.go.source",
            "Go Source Adapter",
            &["go"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold().with_flag("parsing", CapabilitySupport::Partial),
            detect_go,
        ),
        (
            "parallax.java.source",
            "Java Source Adapter",
            &["java"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold(),
            detect_java,
        ),
        (
            "parallax.kotlin.source",
            "Kotlin Source Adapter",
            &["kotlin"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold(),
            detect_kotlin,
        ),
        (
            "parallax.csharp.source",
            "C# Source Adapter",
            &["csharp"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold(),
            detect_csharp,
        ),
        (
            "parallax.ruby.source",
            "Ruby Source Adapter",
            &["ruby"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold().with_flag("parsing", CapabilitySupport::Partial),
            detect_ruby,
        ),
        (
            "parallax.php.source",
            "PHP Source Adapter",
            &["php"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold(),
            detect_php,
        ),
        (
            "parallax.swift.source",
            "Swift Source Adapter",
            &["swift"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_swift,
        ),
        (
            "parallax.dart.source",
            "Dart Source Adapter",
            &["dart"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_dart,
        ),
        (
            "parallax.c.source",
            "C Source Adapter",
            &["c"],
            AdapterMaturity::ParseOnly,
            AdapterCapabilities::scaffold(),
            detect_c,
        ),
        (
            "parallax.cpp.source",
            "C++ Source Adapter",
            &["cpp"],
            AdapterMaturity::ParseOnly,
            AdapterCapabilities::scaffold(),
            detect_cpp,
        ),
        (
            "parallax.lua.source",
            "Lua Source Adapter",
            &["lua"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_lua,
        ),
    ] {
        let mut m =
            AdapterManifest::builtin(id, name, AdapterKind::SourceLanguage, maturity, langs);
        m.owns = vec![
            "ast".into(),
            "symbols".into(),
            "types".into(),
            "control_flow".into(),
        ];
        if id.contains("typescript") {
            m.priority = 30;
        } else if id.contains("javascript") {
            m.priority = 20;
        }
        add(reg, m, caps, detect);
    }

    // Target
    for (id, name, langs, maturity, caps, detect) in [
        (
            "parallax.rust.target",
            "Rust Target Adapter",
            &["rust"][..],
            AdapterMaturity::Stable,
            AdapterCapabilities::rust_target(),
            detect_rust_target as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.go.target",
            "Go Target Adapter",
            &["go"],
            AdapterMaturity::Beta,
            AdapterCapabilities::go_target(),
            detect_go_target,
        ),
        (
            "parallax.python.target",
            "Python Target Adapter",
            &["python"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold().with_flag("codegen", CapabilitySupport::Partial),
            detect_python_target,
        ),
        (
            "parallax.typescript.target",
            "TypeScript Target Adapter",
            &["typescript"],
            AdapterMaturity::Experimental,
            AdapterCapabilities::scaffold(),
            detect_typescript_target,
        ),
        (
            "parallax.java.target",
            "Java Target Adapter",
            &["java"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_java_target,
        ),
        (
            "parallax.kotlin.target",
            "Kotlin Target Adapter",
            &["kotlin"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_kotlin_target,
        ),
        (
            "parallax.csharp.target",
            "C# Target Adapter",
            &["csharp"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_csharp_target,
        ),
        (
            "parallax.ruby.target",
            "Ruby Target Adapter",
            &["ruby"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_ruby_target,
        ),
        (
            "parallax.swift.target",
            "Swift Target Adapter",
            &["swift"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_swift_target,
        ),
        (
            "parallax.dart.target",
            "Dart Target Adapter",
            &["dart"],
            AdapterMaturity::Scaffold,
            AdapterCapabilities::scaffold(),
            detect_dart_target,
        ),
    ] {
        let mut m =
            AdapterManifest::builtin(id, name, AdapterKind::TargetLanguage, maturity, langs);
        m.owns = vec![
            "syntax_emission".into(),
            "module_layout".into(),
            "package_manifest".into(),
        ];
        add(reg, m, caps, detect);
    }

    // Runtimes
    for (id, name, langs, detect) in [
        (
            "parallax.runtime.node",
            "Node.js Runtime Adapter",
            &["javascript", "typescript"][..],
            detect_node as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.runtime.cpython",
            "CPython Runtime Adapter",
            &["python"],
            detect_python,
        ),
        (
            "parallax.runtime.wasm",
            "WebAssembly Runtime Adapter",
            &["wasm"],
            detect_wasm,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            name,
            AdapterKind::Runtime,
            AdapterMaturity::Stable,
            langs,
        );
        m.priority = 15;
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_frameworks(reg: &mut AdapterRegistry) {
    #[allow(clippy::type_complexity)]
    type Fw = (
        &'static str,
        &'static str,
        &'static [&'static str],
        i32,
        fn(&ProjectContext) -> DetectionResult,
    );
    let frameworks: &[Fw] = &[
        (
            "parallax.framework.express",
            "Express",
            &["typescript", "javascript"],
            85,
            detect_express,
        ),
        (
            "parallax.framework.fastify",
            "Fastify",
            &["typescript", "javascript"],
            86,
            detect_fastify,
        ),
        (
            "parallax.framework.nestjs",
            "NestJS",
            &["typescript"],
            95,
            detect_nestjs,
        ),
        (
            "parallax.framework.fastapi",
            "FastAPI",
            &["python"],
            90,
            detect_fastapi,
        ),
        (
            "parallax.framework.flask",
            "Flask",
            &["python"],
            80,
            detect_flask,
        ),
        (
            "parallax.framework.django",
            "Django",
            &["python"],
            88,
            detect_django,
        ),
        (
            "parallax.framework.axum",
            "Axum",
            &["rust"],
            90,
            detect_axum,
        ),
        (
            "parallax.framework.actix",
            "Actix Web",
            &["rust"],
            85,
            detect_actix,
        ),
        ("parallax.framework.gin", "Gin", &["go"], 80, detect_gin),
        ("parallax.framework.chi", "Chi", &["go"], 82, detect_chi),
        (
            "parallax.framework.spring",
            "Spring Boot",
            &["java", "kotlin"],
            90,
            detect_spring,
        ),
        (
            "parallax.framework.aspnet",
            "ASP.NET",
            &["csharp"],
            90,
            detect_aspnet,
        ),
        (
            "parallax.framework.rails",
            "Ruby on Rails",
            &["ruby"],
            88,
            detect_rails,
        ),
        (
            "parallax.framework.laravel",
            "Laravel",
            &["php"],
            85,
            detect_laravel,
        ),
        (
            "parallax.framework.nextjs",
            "Next.js",
            &["typescript", "javascript"],
            92,
            detect_nextjs,
        ),
        (
            "parallax.framework.hono",
            "Hono",
            &["typescript", "javascript"],
            84,
            detect_hono,
        ),
        (
            "parallax.framework.koa",
            "Koa",
            &["typescript", "javascript"],
            83,
            detect_koa,
        ),
        (
            "parallax.framework.fiber",
            "Fiber",
            &["go"],
            88,
            detect_fiber,
        ),
        ("parallax.framework.echo", "Echo", &["go"], 86, detect_echo),
        (
            "parallax.framework.rocket",
            "Rocket",
            &["rust"],
            88,
            detect_rocket,
        ),
        (
            "parallax.framework.ktor",
            "Ktor",
            &["kotlin"],
            87,
            detect_ktor,
        ),
        (
            "parallax.framework.vapor",
            "Vapor",
            &["swift"],
            85,
            detect_vapor,
        ),
        (
            "parallax.framework.litestar",
            "Litestar",
            &["python"],
            86,
            detect_litestar,
        ),
        (
            "parallax.framework.sanic",
            "Sanic",
            &["python"],
            82,
            detect_sanic,
        ),
        (
            "parallax.framework.phoenix",
            "Phoenix",
            &["elixir"],
            90,
            detect_phoenix,
        ),
        (
            "parallax.framework.sinatra",
            "Sinatra",
            &["ruby"],
            80,
            detect_sinatra,
        ),
        (
            "parallax.framework.quarkus",
            "Quarkus",
            &["java", "kotlin"],
            88,
            detect_quarkus,
        ),
        (
            "parallax.framework.micronaut",
            "Micronaut",
            &["java", "kotlin"],
            86,
            detect_micronaut,
        ),
        (
            "parallax.framework.symfony",
            "Symfony",
            &["php"],
            87,
            detect_symfony,
        ),
        ("parallax.framework.slim", "Slim", &["php"], 78, detect_slim),
        (
            "parallax.framework.beego",
            "Beego",
            &["go"],
            79,
            detect_beego,
        ),
        (
            "parallax.framework.buffalo",
            "Buffalo",
            &["go"],
            77,
            detect_buffalo,
        ),
    ];
    for (id, name, langs, prio, detect) in frameworks {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Framework Adapter"),
            AdapterKind::Framework,
            framework_maturity(id),
            langs,
        );
        m.priority = *prio;
        m.owns = vec![
            "routes".into(),
            "middleware".into(),
            "http_lifecycle".into(),
        ];
        m.ecosystems = vec![name.to_lowercase()];
        m.notes = "Preferred mappings via LanguagePairProfile / MigrationPack".to_string();
        add(reg, m, AdapterCapabilities::framework_http(), *detect);
    }

    // React as web-frontend so it can compose with Express/NestJS.
    let mut react = AdapterManifest::builtin(
        "parallax.frontend.react",
        "React Frontend Adapter",
        AdapterKind::WebFrontend,
        AdapterMaturity::Experimental,
        &["typescript", "javascript"],
    );
    react.priority = 70;
    react.owns = vec!["components".into(), "props".into(), "hooks".into()];
    add(
        reg,
        react,
        AdapterCapabilities::scaffold().with_flag("components", CapabilitySupport::Partial),
        detect_react,
    );

    for (id, name, pkg, detect) in [
        (
            "parallax.frontend.vue",
            "Vue",
            "vue",
            detect_vue as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.frontend.svelte",
            "Svelte",
            "svelte",
            detect_svelte,
        ),
        ("parallax.frontend.solid", "Solid", "solid-js", detect_solid),
        (
            "parallax.frontend.angular",
            "Angular",
            "@angular/core",
            detect_angular,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Frontend Adapter"),
            AdapterKind::WebFrontend,
            AdapterMaturity::Experimental,
            &["typescript", "javascript"],
        );
        m.priority = 70;
        m.owns = vec!["components".into(), "templates".into()];
        m.ecosystems = vec![pkg.to_string()];
        add(
            reg,
            m,
            AdapterCapabilities::scaffold().with_flag("components", CapabilitySupport::Partial),
            detect,
        );
    }
}

fn framework_maturity(id: &str) -> AdapterMaturity {
    match id {
        "parallax.framework.express" | "parallax.framework.axum" | "parallax.framework.fastapi" => {
            AdapterMaturity::Stable
        }
        "parallax.framework.flask"
        | "parallax.framework.django"
        | "parallax.framework.gin"
        | "parallax.framework.chi"
        | "parallax.framework.nestjs"
        | "parallax.framework.fastify"
        | "parallax.framework.hono"
        | "parallax.framework.koa"
        | "parallax.framework.fiber"
        | "parallax.framework.echo"
        | "parallax.framework.rocket"
        | "parallax.framework.litestar" => AdapterMaturity::Beta,
        _ => AdapterMaturity::Experimental,
    }
}

fn register_build(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.build.npm",
            "npm / package.json",
            &["javascript", "typescript"][..],
            detect_npm as fn(&ProjectContext) -> DetectionResult,
        ),
        ("parallax.build.cargo", "Cargo", &["rust"], detect_cargo),
        (
            "parallax.build.pip",
            "pip / pyproject",
            &["python"],
            detect_pip,
        ),
        ("parallax.build.gomod", "Go Modules", &["go"], detect_gomod),
        ("parallax.build.maven", "Maven", &["java"], detect_maven),
        (
            "parallax.build.gradle",
            "Gradle",
            &["java", "kotlin"],
            detect_gradle,
        ),
        (
            "parallax.build.msbuild",
            "MSBuild / .csproj",
            &["csharp"],
            detect_csproj,
        ),
        (
            "parallax.build.bundler",
            "Bundler / Gemfile",
            &["ruby"],
            detect_gemfile,
        ),
        (
            "parallax.build.composer",
            "Composer",
            &["php"],
            detect_composer,
        ),
        (
            "parallax.build.swiftpm",
            "SwiftPM",
            &["swift"],
            detect_swiftpm,
        ),
        ("parallax.build.pub", "Dart pub", &["dart"], detect_pubspec),
        (
            "parallax.build.pnpm",
            "pnpm",
            &["javascript", "typescript"],
            detect_pnpm,
        ),
        (
            "parallax.build.yarn",
            "Yarn",
            &["javascript", "typescript"],
            detect_yarn,
        ),
        (
            "parallax.build.bun",
            "Bun",
            &["javascript", "typescript"],
            detect_bun,
        ),
        ("parallax.build.uv", "uv", &["python"], detect_uv),
        (
            "parallax.build.poetry",
            "Poetry",
            &["python"],
            detect_poetry,
        ),
        ("parallax.build.cmake", "CMake", &["c", "cpp"], detect_cmake),
        ("parallax.build.meson", "Meson", &["c", "cpp"], detect_meson),
        ("parallax.build.sbt", "sbt", &["scala"], detect_sbt),
        (
            "parallax.build.lein",
            "Leiningen",
            &["clojure"],
            detect_lein,
        ),
    ] {
        let m = AdapterManifest::builtin(
            id,
            name,
            AdapterKind::BuildSystem,
            AdapterMaturity::Beta,
            langs,
        );
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_tests(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.test.jest",
            "Jest",
            &["javascript", "typescript"][..],
            detect_jest as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.test.vitest",
            "Vitest",
            &["javascript", "typescript"],
            detect_vitest,
        ),
        ("parallax.test.pytest", "pytest", &["python"], detect_pytest),
        (
            "parallax.test.cargo",
            "Rust libtest",
            &["rust"],
            detect_cargo_test,
        ),
        ("parallax.test.go", "Go testing", &["go"], detect_go_test),
        (
            "parallax.test.junit",
            "JUnit",
            &["java", "kotlin"],
            detect_junit,
        ),
        ("parallax.test.xunit", "xUnit", &["csharp"], detect_xunit),
        ("parallax.test.rspec", "RSpec", &["ruby"], detect_rspec),
        ("parallax.test.phpunit", "PHPUnit", &["php"], detect_phpunit),
        (
            "parallax.test.mocha",
            "Mocha",
            &["javascript", "typescript"],
            detect_mocha,
        ),
        (
            "parallax.test.criterion",
            "Criterion",
            &["rust"],
            detect_criterion,
        ),
        ("parallax.test.kotest", "Kotest", &["kotlin"], detect_kotest),
        ("parallax.test.nunit", "NUnit", &["csharp"], detect_nunit),
        ("parallax.test.xctest", "XCTest", &["swift"], detect_xctest),
        (
            "parallax.test.dart",
            "Dart test",
            &["dart"],
            detect_dart_test,
        ),
        (
            "parallax.test.unittest",
            "unittest",
            &["python"],
            detect_unittest,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Test Adapter"),
            AdapterKind::TestFramework,
            AdapterMaturity::Beta,
            langs,
        );
        m.owns = vec!["test_cases".into(), "assertions".into(), "fixtures".into()];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_data(reg: &mut AdapterRegistry) {
    for (id, name, kind, detect) in [
        (
            "parallax.db.postgres",
            "PostgreSQL",
            AdapterKind::Database,
            detect_postgres as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.db.mysql",
            "MySQL",
            AdapterKind::Database,
            detect_mysql,
        ),
        (
            "parallax.db.sqlite",
            "SQLite",
            AdapterKind::Database,
            detect_sqlite,
        ),
        (
            "parallax.db.mongodb",
            "MongoDB",
            AdapterKind::Database,
            detect_mongodb,
        ),
        (
            "parallax.db.redis",
            "Redis",
            AdapterKind::Database,
            detect_redis,
        ),
        (
            "parallax.orm.prisma",
            "Prisma",
            AdapterKind::Orm,
            detect_prisma,
        ),
        (
            "parallax.orm.sqlalchemy",
            "SQLAlchemy",
            AdapterKind::Orm,
            detect_sqlalchemy,
        ),
        (
            "parallax.orm.diesel",
            "Diesel",
            AdapterKind::Orm,
            detect_diesel,
        ),
        ("parallax.orm.sqlx", "SQLx", AdapterKind::Orm, detect_sqlx),
        (
            "parallax.orm.hibernate",
            "Hibernate",
            AdapterKind::Orm,
            detect_hibernate,
        ),
        (
            "parallax.orm.efcore",
            "Entity Framework Core",
            AdapterKind::Orm,
            detect_efcore,
        ),
        (
            "parallax.orm.activerecord",
            "ActiveRecord",
            AdapterKind::Orm,
            detect_activerecord,
        ),
        (
            "parallax.orm.drizzle",
            "Drizzle",
            AdapterKind::Orm,
            detect_drizzle,
        ),
        (
            "parallax.orm.seaorm",
            "SeaORM",
            AdapterKind::Orm,
            detect_seaorm,
        ),
        ("parallax.orm.gorm", "GORM", AdapterKind::Orm, detect_gorm),
        (
            "parallax.orm.eloquent",
            "Eloquent",
            AdapterKind::Orm,
            detect_eloquent,
        ),
        (
            "parallax.db.dynamodb",
            "DynamoDB",
            AdapterKind::Database,
            detect_dynamodb,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Adapter"),
            kind,
            AdapterMaturity::Experimental,
            &[],
        );
        m.ecosystems = vec![name.to_lowercase()];
        m.owns = if matches!(kind, AdapterKind::Orm) {
            vec!["models".into(), "queries".into(), "migrations".into()]
        } else {
            vec!["connection".into(), "transactions".into()]
        };
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_deploy(reg: &mut AdapterRegistry) {
    for (id, name, detect) in [
        (
            "parallax.deploy.docker",
            "Docker",
            detect_docker as fn(&ProjectContext) -> DetectionResult,
        ),
        ("parallax.deploy.compose", "Docker Compose", detect_compose),
        ("parallax.deploy.k8s", "Kubernetes", detect_k8s),
        (
            "parallax.deploy.github_actions",
            "GitHub Actions",
            detect_gha,
        ),
        ("parallax.deploy.vercel", "Vercel", detect_vercel),
        ("parallax.deploy.render", "Render", detect_render),
        ("parallax.deploy.fly", "Fly.io", detect_fly),
        ("parallax.deploy.railway", "Railway", detect_railway),
        ("parallax.deploy.netlify", "Netlify", detect_netlify),
        ("parallax.deploy.gitlab_ci", "GitLab CI", detect_gitlab_ci),
        ("parallax.deploy.circleci", "CircleCI", detect_circleci),
        (
            "parallax.deploy.aws_lambda",
            "AWS Lambda",
            detect_aws_lambda,
        ),
    ] {
        let m = AdapterManifest::builtin(
            id,
            &format!("{name} Deployment Adapter"),
            AdapterKind::Deployment,
            AdapterMaturity::Experimental,
            &[],
        );
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_cli(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.cli.clap",
            "clap",
            &["rust"][..],
            detect_clap as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.cli.commander",
            "commander",
            &["javascript", "typescript"],
            detect_commander,
        ),
        ("parallax.cli.cobra", "cobra", &["go"], detect_cobra),
        ("parallax.cli.click", "click", &["python"], detect_click),
        ("parallax.cli.typer", "typer", &["python"], detect_typer),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} CLI Adapter"),
            AdapterKind::CliFramework,
            AdapterMaturity::Experimental,
            langs,
        );
        m.owns = vec!["commands".into(), "flags".into(), "subcommands".into()];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_serialization(reg: &mut AdapterRegistry) {
    for (id, name, kind, langs, detect) in [
        (
            "parallax.validation.zod",
            "Zod",
            AdapterKind::Validation,
            &["typescript", "javascript"][..],
            detect_zod as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.validation.pydantic",
            "Pydantic",
            AdapterKind::Validation,
            &["python"],
            detect_pydantic,
        ),
        (
            "parallax.serialization.serde",
            "Serde",
            AdapterKind::Serialization,
            &["rust"],
            detect_serde,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Adapter"),
            kind,
            if id.contains("serde") {
                AdapterMaturity::Stable
            } else {
                AdapterMaturity::Beta
            },
            langs,
        );
        m.owns = vec![
            "schemas".into(),
            "validation".into(),
            "serialization".into(),
        ];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_formatters(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.formatter.prettier",
            "Prettier",
            &["javascript", "typescript"][..],
            detect_prettier as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.formatter.biome",
            "Biome",
            &["javascript", "typescript"],
            detect_biome,
        ),
        (
            "parallax.formatter.rustfmt",
            "rustfmt",
            &["rust"],
            detect_rustfmt,
        ),
        (
            "parallax.formatter.black",
            "Black",
            &["python"],
            detect_black,
        ),
        (
            "parallax.formatter.ruff",
            "Ruff format",
            &["python"],
            detect_ruff_format,
        ),
        ("parallax.formatter.gofmt", "gofmt", &["go"], detect_gofmt),
        (
            "parallax.formatter.dart",
            "dart format",
            &["dart"],
            detect_dart_format,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Formatter Adapter"),
            AdapterKind::Formatter,
            AdapterMaturity::Beta,
            langs,
        );
        m.owns = vec!["style".into(), "formatting".into()];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_linters(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.linter.eslint",
            "ESLint",
            &["javascript", "typescript"][..],
            detect_eslint as fn(&ProjectContext) -> DetectionResult,
        ),
        ("parallax.linter.clippy", "Clippy", &["rust"], detect_clippy),
        (
            "parallax.linter.ruff",
            "Ruff lint",
            &["python"],
            detect_ruff_lint,
        ),
        (
            "parallax.linter.pylint",
            "Pylint",
            &["python"],
            detect_pylint,
        ),
        (
            "parallax.linter.golangci",
            "golangci-lint",
            &["go"],
            detect_golangci,
        ),
        (
            "parallax.linter.rubocop",
            "RuboCop",
            &["ruby"],
            detect_rubocop,
        ),
        ("parallax.linter.mypy", "mypy", &["python"], detect_mypy),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Linter Adapter"),
            AdapterKind::Linter,
            AdapterMaturity::Experimental,
            langs,
        );
        m.owns = vec!["diagnostics".into(), "rules".into()];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_codegen(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.codegen.openapi",
            "OpenAPI / Swagger",
            &["typescript", "javascript", "python", "go", "java"][..],
            detect_openapi as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.codegen.protobuf",
            "Protocol Buffers",
            &["go", "java", "python", "csharp", "typescript"],
            detect_protobuf,
        ),
        (
            "parallax.codegen.graphql",
            "GraphQL Codegen",
            &["typescript", "javascript"],
            detect_graphql_codegen,
        ),
        (
            "parallax.codegen.openapi_generator",
            "OpenAPI Generator",
            &["java", "typescript", "go", "python"],
            detect_openapi_generator,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Codegen Adapter"),
            AdapterKind::Codegen,
            AdapterMaturity::Experimental,
            langs,
        );
        m.owns = vec![
            "schemas".into(),
            "generated_clients".into(),
            "generated_servers".into(),
        ];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_desktop_gui(reg: &mut AdapterRegistry) {
    for (id, name, langs, detect) in [
        (
            "parallax.desktop.tauri",
            "Tauri",
            &["rust", "typescript", "javascript"][..],
            detect_tauri as fn(&ProjectContext) -> DetectionResult,
        ),
        (
            "parallax.desktop.electron",
            "Electron",
            &["javascript", "typescript"],
            detect_electron,
        ),
        (
            "parallax.desktop.wails",
            "Wails",
            &["go", "javascript", "typescript"],
            detect_wails,
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            &format!("{name} Desktop GUI Adapter"),
            AdapterKind::DesktopGui,
            AdapterMaturity::Experimental,
            langs,
        );
        m.owns = vec!["native_shell".into(), "webview".into(), "ipc".into()];
        add(reg, m, AdapterCapabilities::scaffold(), detect);
    }
}

fn register_pairs(reg: &mut AdapterRegistry) {
    for (id, name, langs) in [
        (
            "parallax.pair.typescript-rust",
            "TypeScript → Rust pair profile",
            &["typescript", "rust"][..],
        ),
        (
            "parallax.pair.python-rust",
            "Python → Rust pair profile",
            &["python", "rust"],
        ),
        (
            "parallax.pair.typescript-go",
            "TypeScript → Go pair profile",
            &["typescript", "go"],
        ),
        (
            "parallax.pair.java-kotlin",
            "Java → Kotlin pair profile",
            &["java", "kotlin"],
        ),
        (
            "parallax.pair.go-rust",
            "Go → Rust pair profile",
            &["go", "rust"],
        ),
    ] {
        let mut m = AdapterManifest::builtin(
            id,
            name,
            AdapterKind::PairProfile,
            AdapterMaturity::Beta,
            langs,
        );
        m.priority = 5;
        add(reg, m, AdapterCapabilities::scaffold(), |_| {
            DetectionResult::no_match()
        });
    }
}

// ── detectors ─────────────────────────────────────────────────────────

macro_rules! detect_ext {
    ($ctx:expr, $ext:expr, $conf:expr) => {{
        if $ctx.has_file_suffix($ext) {
            DetectionResult::matched($conf).evidence("extension", $ext)
        } else {
            DetectionResult::no_match()
        }
    }};
}

macro_rules! detect_pkg {
    ($ctx:expr, $pkg:expr, $conf:expr) => {{
        if $ctx.package_contains($pkg) {
            DetectionResult::matched($conf).evidence("package", $pkg)
        } else {
            DetectionResult::no_match()
        }
    }};
}

fn detect_typescript(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".ts")
        || ctx.has_file_suffix(".tsx")
        || ctx.has_manifest("tsconfig.json")
    {
        DetectionResult::matched(DetectionConfidence::High)
            .evidence("language", "typescript")
            .owns(&["ast", "symbols", "types"])
    } else {
        DetectionResult::no_match()
    }
}
fn detect_javascript(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".js") || ctx.has_file_suffix(".mjs") || ctx.has_manifest("package.json")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("language", "javascript")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_python(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".py")
        || ctx.has_manifest("pyproject.toml")
        || ctx.has_manifest("requirements.txt")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "python")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_go(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".go") || ctx.has_manifest("go.mod") {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "go")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_java(ctx: &ProjectContext) -> DetectionResult {
    detect_ext!(ctx, ".java", DetectionConfidence::High)
}
fn detect_kotlin(ctx: &ProjectContext) -> DetectionResult {
    detect_ext!(ctx, ".kt", DetectionConfidence::High)
}
fn detect_csharp(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".cs") || ctx.has_file_suffix(".csproj") {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "csharp")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_ruby(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".rb") || ctx.has_manifest("Gemfile") {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "ruby")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_php(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".php") || ctx.has_manifest("composer.json") {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "php")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_swift(ctx: &ProjectContext) -> DetectionResult {
    detect_ext!(ctx, ".swift", DetectionConfidence::Medium)
}
fn detect_dart(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".dart") || ctx.has_manifest("pubspec.yaml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("language", "dart")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_c(ctx: &ProjectContext) -> DetectionResult {
    detect_ext!(ctx, ".c", DetectionConfidence::Medium)
}
fn detect_cpp(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".cpp") || ctx.has_file_suffix(".cc") || ctx.has_file_suffix(".cxx") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("language", "cpp")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_lua(ctx: &ProjectContext) -> DetectionResult {
    detect_ext!(ctx, ".lua", DetectionConfidence::Medium)
}
fn detect_wasm(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".wasm") || ctx.has_file_suffix(".wat") {
        DetectionResult::matched(DetectionConfidence::High).evidence("runtime", "wasm")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_node(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("package.json") {
        DetectionResult::matched(DetectionConfidence::High).evidence("runtime", "node")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_rust_target(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Cargo.toml")
        || ctx
            .hints
            .get("target")
            .map(|t| t == "rust")
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("target", "rust")
    } else if ctx.hints.get("to").map(|t| t == "rust").unwrap_or(false) {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("cli", "--to rust")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_go_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "go")
}
fn detect_python_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "python")
}
fn detect_typescript_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "typescript")
}
fn detect_java_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "java")
}
fn detect_kotlin_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "kotlin")
}
fn detect_csharp_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "csharp")
}
fn detect_ruby_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "ruby")
}
fn detect_swift_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "swift")
}
fn detect_dart_target(ctx: &ProjectContext) -> DetectionResult {
    target_hint(ctx, "dart")
}

fn target_hint(ctx: &ProjectContext, lang: &str) -> DetectionResult {
    if ctx.hints.get("to").map(|t| t == lang).unwrap_or(false) {
        DetectionResult::matched(DetectionConfidence::Certain)
            .evidence("cli", format!("--to {lang}"))
    } else {
        DetectionResult::no_match()
    }
}

fn detect_express(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "express", DetectionConfidence::High)
}
fn detect_fastify(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "fastify", DetectionConfidence::High)
}
fn detect_nestjs(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("@nestjs") || ctx.package_contains("nestjs") {
        DetectionResult::matched(DetectionConfidence::Certain)
            .evidence("package", "@nestjs/core")
            .owns(&["routes", "middleware", "providers"])
    } else {
        DetectionResult::no_match()
    }
}
fn detect_fastapi(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "fastapi", DetectionConfidence::High)
}
fn detect_flask(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "flask", DetectionConfidence::High)
}
fn detect_django(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "django", DetectionConfidence::High)
}
fn detect_axum(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "axum", DetectionConfidence::High)
}
fn detect_actix(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "actix-web", DetectionConfidence::High)
}
fn detect_gin(ctx: &ProjectContext) -> DetectionResult {
    if ctx.files.iter().any(|f| f.contains("gin-gonic")) || ctx.package_contains("gin-gonic/gin") {
        DetectionResult::matched(DetectionConfidence::High).evidence("module", "gin")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_chi(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "go-chi/chi", DetectionConfidence::High)
}
fn detect_spring(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("spring-boot") || ctx.has_file_suffix("Application.java") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "spring")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_aspnet(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("Microsoft.AspNetCore") || ctx.has_file_suffix("Startup.cs") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "aspnet")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_rails(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Gemfile")
        && (ctx.package_contains("rails") || ctx.has_file_suffix("config/routes.rb"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "rails")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_laravel(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("laravel/framework") || ctx.has_file_suffix("artisan") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "laravel")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_nextjs(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "next", DetectionConfidence::High)
}
fn detect_react(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "react", DetectionConfidence::Medium)
}
fn detect_vue(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "vue", DetectionConfidence::Medium)
}
fn detect_svelte(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "svelte", DetectionConfidence::Medium)
}
fn detect_solid(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("solid-js") || ctx.package_contains("solidjs") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("package", "solid-js")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_angular(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("@angular/core") || ctx.package_contains("angular") {
        DetectionResult::matched(DetectionConfidence::High).evidence("package", "@angular/core")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_hono(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "hono", DetectionConfidence::High)
}
fn detect_koa(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "koa", DetectionConfidence::High)
}
fn detect_fiber(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "gofiber/fiber", DetectionConfidence::High)
}
fn detect_echo(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "labstack/echo", DetectionConfidence::High)
}
fn detect_rocket(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "rocket", DetectionConfidence::High)
}
fn detect_ktor(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("ktor") || ctx.package_contains("io.ktor") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "ktor")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_vapor(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("vapor")
        || ctx
            .manifests
            .get("Package.swift")
            .map(|t| t.to_ascii_lowercase().contains("vapor"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("framework", "vapor")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_litestar(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "litestar", DetectionConfidence::High)
}
fn detect_sanic(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sanic", DetectionConfidence::High)
}
fn detect_phoenix(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("mix.exs") && ctx.package_contains("phoenix") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "phoenix")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_sinatra(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sinatra", DetectionConfidence::High)
}

fn detect_npm(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("package.json") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "package.json")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_cargo(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Cargo.toml") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "Cargo.toml")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_pip(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pyproject.toml")
        || ctx.has_manifest("requirements.txt")
        || ctx.has_manifest("Pipfile")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("manifest", "python-deps")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gomod(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("go.mod") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "go.mod")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_maven(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pom.xml") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "pom.xml")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gradle(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("build.gradle") || ctx.has_manifest("build.gradle.kts") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "gradle")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_csproj(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".csproj") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "csproj")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gemfile(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Gemfile") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "Gemfile")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_composer(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("composer.json") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "composer.json")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_swiftpm(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Package.swift") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "Package.swift")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_pubspec(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pubspec.yaml") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "pubspec.yaml")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_pnpm(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pnpm-lock.yaml")
        || ctx
            .manifests
            .get("package.json")
            .map(|t| t.contains("pnpm"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("manifest", "pnpm")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_yarn(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("yarn.lock") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "yarn.lock")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_bun(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("bun.lockb") || ctx.has_manifest("bunfig.toml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("manifest", "bun")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_uv(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("uv.lock")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.uv]"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("manifest", "uv")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_poetry(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("poetry.lock")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.poetry]"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("manifest", "poetry")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_cmake(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("CMakeLists.txt") {
        DetectionResult::matched(DetectionConfidence::Certain)
            .evidence("manifest", "CMakeLists.txt")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_meson(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("meson.build") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "meson.build")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_jest(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "jest", DetectionConfidence::High)
}
fn detect_vitest(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "vitest", DetectionConfidence::High)
}
fn detect_pytest(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("pytest")
        || ctx
            .files
            .iter()
            .any(|f| f.contains("test_") && f.ends_with(".py"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("tests", "pytest")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_cargo_test(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Cargo.toml") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("tests", "cargo test")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_go_test(ctx: &ProjectContext) -> DetectionResult {
    if ctx.files.iter().any(|f| f.ends_with("_test.go")) {
        DetectionResult::matched(DetectionConfidence::High).evidence("tests", "go test")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_junit(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "junit", DetectionConfidence::Medium)
}
fn detect_xunit(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "xunit", DetectionConfidence::Medium)
}
fn detect_rspec(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "rspec", DetectionConfidence::High)
}
fn detect_phpunit(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "phpunit", DetectionConfidence::High)
}
fn detect_mocha(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "mocha", DetectionConfidence::High)
}
fn detect_criterion(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "criterion", DetectionConfidence::High)
}
fn detect_kotest(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("kotest") || ctx.package_contains("io.kotest") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("tests", "kotest")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_nunit(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "nunit", DetectionConfidence::Medium)
}
fn detect_xctest(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Package.swift")
        && ctx
            .files
            .iter()
            .any(|f| f.contains("Tests") && f.ends_with(".swift"))
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("tests", "xctest")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_dart_test(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pubspec.yaml")
        && ctx
            .files
            .iter()
            .any(|f| f.ends_with("_test.dart") || f.contains("/test/"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("tests", "dart test")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_unittest(ctx: &ProjectContext) -> DetectionResult {
    if ctx
        .files
        .iter()
        .any(|f| f.contains("test_") && f.ends_with(".py"))
        && !ctx.package_contains("pytest")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("tests", "unittest")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_postgres(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("postgres")
        || ctx.package_contains("pg")
        || ctx.package_contains("psycopg")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("database", "postgres")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_mysql(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "mysql", DetectionConfidence::Medium)
}
fn detect_sqlite(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sqlite", DetectionConfidence::Medium)
}
fn detect_mongodb(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "mongo", DetectionConfidence::Medium)
}
fn detect_redis(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "redis", DetectionConfidence::Medium)
}
fn detect_prisma(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("prisma") || ctx.has_file_suffix("schema.prisma") {
        DetectionResult::matched(DetectionConfidence::High)
            .evidence("orm", "prisma")
            .owns(&["models", "queries", "migrations"])
    } else {
        DetectionResult::no_match()
    }
}
fn detect_sqlalchemy(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sqlalchemy", DetectionConfidence::High)
}
fn detect_diesel(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "diesel", DetectionConfidence::High)
}
fn detect_sqlx(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sqlx", DetectionConfidence::High)
}
fn detect_hibernate(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "hibernate", DetectionConfidence::Medium)
}
fn detect_efcore(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "EntityFrameworkCore", DetectionConfidence::High)
}
fn detect_activerecord(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("activerecord") || ctx.package_contains("rails") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("orm", "activerecord")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_drizzle(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("drizzle-orm") || ctx.package_contains("drizzle-kit") {
        DetectionResult::matched(DetectionConfidence::High).evidence("orm", "drizzle")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_seaorm(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "sea-orm", DetectionConfidence::High)
}
fn detect_gorm(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "gorm.io/gorm", DetectionConfidence::High)
}
fn detect_eloquent(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("laravel/framework") || ctx.has_file_suffix("artisan") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("orm", "eloquent")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_dynamodb(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("dynamodb")
        || ctx.package_contains("aws-sdk")
        || ctx.package_contains("@aws-sdk/client-dynamodb")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("database", "dynamodb")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_docker(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Dockerfile") || ctx.files.iter().any(|f| f.ends_with("Dockerfile")) {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "docker")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_compose(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("docker-compose.yml")
        || ctx.has_manifest("docker-compose.yaml")
        || ctx.has_manifest("compose.yaml")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "compose")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_k8s(ctx: &ProjectContext) -> DetectionResult {
    if ctx
        .files
        .iter()
        .any(|f| f.contains("k8s/") || f.contains("kubernetes/"))
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("deploy", "k8s")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gha(ctx: &ProjectContext) -> DetectionResult {
    if ctx.files.iter().any(|f| f.contains(".github/workflows/")) {
        DetectionResult::matched(DetectionConfidence::High).evidence("ci", "github-actions")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_vercel(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("vercel.json") {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "vercel")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_render(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("render.yaml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "render")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_fly(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("fly.toml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "fly.io")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_railway(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("railway.toml") || ctx.has_manifest("railway.json") {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "railway")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_netlify(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("netlify.toml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("deploy", "netlify")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gitlab_ci(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest(".gitlab-ci.yml") {
        DetectionResult::matched(DetectionConfidence::High).evidence("ci", "gitlab-ci")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_circleci(ctx: &ProjectContext) -> DetectionResult {
    if ctx.files.iter().any(|f| f.contains(".circleci/")) {
        DetectionResult::matched(DetectionConfidence::High).evidence("ci", "circleci")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_aws_lambda(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("serverless.yml")
        || ctx.has_manifest("serverless.yaml")
        || ctx.has_manifest("template.yaml")
        || ctx.has_manifest("samconfig.toml")
        || ctx.package_contains("serverless")
        || ctx.package_contains("@pulumi/aws")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("deploy", "aws-lambda")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_clap(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "clap", DetectionConfidence::High)
}
fn detect_commander(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "commander", DetectionConfidence::High)
}
fn detect_cobra(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "cobra", DetectionConfidence::High)
}
fn detect_click(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "click", DetectionConfidence::High)
}
fn detect_typer(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "typer", DetectionConfidence::High)
}
fn detect_zod(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "zod", DetectionConfidence::High)
}
fn detect_pydantic(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "pydantic", DetectionConfidence::High)
}
fn detect_serde(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "serde", DetectionConfidence::High)
}

fn detect_prettier(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("prettier")
        || ctx.has_manifest(".prettierrc")
        || ctx.has_manifest("prettier.config.js")
        || ctx.has_manifest("prettier.config.mjs")
        || ctx.has_manifest(".prettierrc.json")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("formatter", "prettier")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_biome(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("@biomejs/biome")
        || ctx.has_manifest("biome.json")
        || ctx.has_manifest("biome.jsonc")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("formatter", "biome")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_rustfmt(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("rustfmt.toml")
        || ctx
            .manifests
            .get("Cargo.toml")
            .map(|t| t.contains("[workspace.metadata.rustfmt]") || t.contains("rustfmt"))
            .unwrap_or(false)
        || ctx.has_manifest("Cargo.toml")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("formatter", "rustfmt")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_black(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("black")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.black]"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("formatter", "black")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_ruff_format(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("ruff")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.ruff") && t.contains("format"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("formatter", "ruff")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_gofmt(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("go.mod") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("formatter", "gofmt")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_dart_format(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("pubspec.yaml") || ctx.has_manifest("analysis_options.yaml") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("formatter", "dart format")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_eslint(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("eslint")
        || ctx.has_manifest("eslint.config.js")
        || ctx.has_manifest("eslint.config.mjs")
        || ctx.has_manifest(".eslintrc.json")
        || ctx.has_manifest(".eslintrc.js")
        || ctx.has_manifest(".eslintrc.cjs")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("linter", "eslint")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_clippy(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("Cargo.toml") {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("linter", "clippy")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_ruff_lint(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("ruff")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.ruff.lint]") || t.contains("[tool.ruff]"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("linter", "ruff")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_pylint(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "pylint", DetectionConfidence::High)
}
fn detect_golangci(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest(".golangci.yml")
        || ctx.has_manifest(".golangci.yaml")
        || ctx.files.iter().any(|f| f.ends_with(".golangci.yml"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("linter", "golangci-lint")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_rubocop(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("rubocop")
        || ctx.has_manifest(".rubocop.yml")
        || ctx.has_manifest(".rubocop.yaml")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("linter", "rubocop")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_mypy(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("mypy")
        || ctx
            .manifests
            .get("pyproject.toml")
            .map(|t| t.contains("[tool.mypy]"))
            .unwrap_or(false)
        || ctx.has_manifest("mypy.ini")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("linter", "mypy")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_openapi(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix("openapi.yaml")
        || ctx.has_file_suffix("openapi.yml")
        || ctx.has_file_suffix("openapi.json")
        || ctx.has_file_suffix("swagger.yaml")
        || ctx.has_file_suffix("swagger.yml")
        || ctx.has_file_suffix("swagger.json")
        || ctx.package_contains("@nestjs/swagger")
        || ctx.package_contains("swagger-ui-express")
        || ctx.package_contains("fastapi")
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("codegen", "openapi")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_protobuf(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_file_suffix(".proto")
        || ctx.package_contains("protobuf")
        || ctx.package_contains("prost")
        || ctx.package_contains("tonic")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("codegen", "protobuf")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_graphql_codegen(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("@graphql-codegen")
        || ctx.package_contains("graphql-codegen")
        || ctx.has_manifest("codegen.yml")
        || ctx.has_manifest("codegen.ts")
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("codegen", "graphql")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_openapi_generator(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("openapitools.json")
        || ctx.package_contains("@openapitools/openapi-generator-cli")
        || ctx.files.iter().any(|f| f.contains("openapi-generator"))
    {
        DetectionResult::matched(DetectionConfidence::Medium)
            .evidence("codegen", "openapi-generator")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_tauri(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("@tauri-apps/api")
        || ctx.package_contains("tauri")
        || ctx.has_manifest("src-tauri/tauri.conf.json")
        || ctx.has_manifest("tauri.conf.json")
        || ctx.files.iter().any(|f| f.contains("src-tauri/"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("desktop", "tauri")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_electron(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("electron")
        || ctx.has_manifest("electron-builder.yml")
        || ctx.files.iter().any(|f| f.contains("electron-main"))
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("desktop", "electron")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_wails(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("wails")
        || ctx.has_manifest("wails.json")
        || ctx.files.iter().any(|f| f.contains("wails/"))
    {
        DetectionResult::matched(DetectionConfidence::Medium).evidence("desktop", "wails")
    } else {
        DetectionResult::no_match()
    }
}

fn detect_quarkus(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("quarkus")
        || ctx
            .manifests
            .get("pom.xml")
            .map(|t| t.contains("quarkus"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "quarkus")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_micronaut(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("micronaut")
        || ctx
            .manifests
            .get("build.gradle")
            .map(|t| t.contains("micronaut"))
            .unwrap_or(false)
        || ctx
            .manifests
            .get("build.gradle.kts")
            .map(|t| t.contains("micronaut"))
            .unwrap_or(false)
    {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "micronaut")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_symfony(ctx: &ProjectContext) -> DetectionResult {
    if ctx.package_contains("symfony/framework-bundle") || ctx.package_contains("symfony/symfony") {
        DetectionResult::matched(DetectionConfidence::High).evidence("framework", "symfony")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_slim(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "slim/slim", DetectionConfidence::High)
}
fn detect_beego(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "beego/beego", DetectionConfidence::High)
}
fn detect_buffalo(ctx: &ProjectContext) -> DetectionResult {
    detect_pkg!(ctx, "gobuffalo/buffalo", DetectionConfidence::High)
}

fn detect_sbt(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("build.sbt") || ctx.files.iter().any(|f| f.ends_with("build.sbt")) {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "build.sbt")
    } else {
        DetectionResult::no_match()
    }
}
fn detect_lein(ctx: &ProjectContext) -> DetectionResult {
    if ctx.has_manifest("project.clj") {
        DetectionResult::matched(DetectionConfidence::Certain).evidence("manifest", "project.clj")
    } else {
        DetectionResult::no_match()
    }
}
