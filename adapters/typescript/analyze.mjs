#!/usr/bin/env node
/**
 * Parallax TypeScript/JavaScript frontend.
 * Uses the TypeScript compiler API (real parser) to emit ProjectGraph + PUIR modules as JSON.
 *
 * Usage: node analyze.mjs <project-root>
 */
import fs from "node:fs";
import path from "node:path";
import ts from "typescript";

const root = path.resolve(process.argv[2] || ".");
if (!fs.existsSync(root)) {
  console.error(`root not found: ${root}`);
  process.exit(2);
}

let nextId = 1;
const nid = () => nextId++;

function rel(p) {
  return path.relative(root, p).split(path.sep).join("/");
}

function walk(dir, acc = []) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    if (["node_modules", ".git", "dist", "build", "target", ".parallax", "coverage"].includes(ent.name)) {
      continue;
    }
    const full = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(full, acc);
    else if (/\.(ts|tsx|js|jsx|mjs)$/.test(ent.name) && !ent.name.endsWith(".d.ts")) acc.push(full);
  }
  return acc;
}

function loadPackage() {
  const pkgPath = path.join(root, "package.json");
  if (!fs.existsSync(pkgPath)) return { deps: [], name: path.basename(root), scripts: {} };
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  const deps = [];
  for (const [name, version] of Object.entries(pkg.dependencies || {})) {
    deps.push({ ecosystem: "npm", name, version: String(version), dev: false });
  }
  for (const [name, version] of Object.entries(pkg.devDependencies || {})) {
    deps.push({ ecosystem: "npm", name, version: String(version), dev: true });
  }
  return { deps, name: pkg.name || path.basename(root), scripts: pkg.scripts || {}, main: pkg.main };
}

function spanOf(sf, node) {
  const start = sf.getLineAndCharacterOfPosition(node.getStart(sf));
  return {
    file: rel(sf.fileName),
    line: start.line + 1,
    column: start.character + 1,
    end_line: null,
    end_column: null,
  };
}

function mapType(node, sf, checker) {
  if (!node) return { kind: "unknown" };
  const t = checker ? checker.getTypeFromTypeNode?.(node) : null;
  const text = node.getText?.(sf) || "";
  if (text === "string" || text === "String") return { kind: "string" };
  if (text === "boolean" || text === "Boolean") return { kind: "bool" };
  if (text === "number" || text === "Number") return { kind: "float", bits: 64 };
  if (text === "void" || text === "undefined") return { kind: "unit" };
  if (text.endsWith("[]")) {
    const inner = text.slice(0, -2).trim();
    if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(inner) && /^[A-Z]/.test(inner)) {
      return { kind: "list", element: { kind: "named", name: inner, module: null } };
    }
    return { kind: "list", element: { kind: "unknown" } };
  }
  if (text.startsWith("Promise<")) {
    return { kind: "future", output: { kind: "unknown" } };
  }
  if (text.includes("|") && text.includes("null")) {
    return { kind: "optional", inner: { kind: "unknown" } };
  }
  if (/^[A-Z]/.test(text)) return { kind: "named", name: text.split(/[<\s|]/)[0], module: null };
  void t;
  return { kind: "unknown" };
}

function exprFrom(node, sf) {
  if (!node) return { op: "unsupported", id: nid(), original: "<empty>", span: null };
  const sp = spanOf(sf, node);
  switch (node.kind) {
    case ts.SyntaxKind.StringLiteral:
      return { op: "constant", id: nid(), value: node.text, span: sp };
    case ts.SyntaxKind.NumericLiteral:
      return { op: "constant", id: nid(), value: Number(node.text), span: sp };
    case ts.SyntaxKind.TrueKeyword:
      return { op: "constant", id: nid(), value: true, span: sp };
    case ts.SyntaxKind.FalseKeyword:
      return { op: "constant", id: nid(), value: false, span: sp };
    case ts.SyntaxKind.NullKeyword:
      return { op: "constant", id: nid(), value: null, span: sp };
    case ts.SyntaxKind.Identifier:
      return { op: "name", id: nid(), name: node.text, span: sp };
    case ts.SyntaxKind.PropertyAccessExpression:
      return {
        op: "access_field",
        id: nid(),
        object: exprFrom(node.expression, sf),
        field: node.name.getText(sf),
        span: sp,
      };
    case ts.SyntaxKind.CallExpression: {
      const calleeText = node.expression.getText(sf);
      // Intrinsics
      if (calleeText === "JSON.parse") {
        return { op: "intrinsic", id: nid(), name: "json.parse", args: node.arguments.map((a) => exprFrom(a, sf)), span: sp };
      }
      if (calleeText === "JSON.stringify") {
        return { op: "intrinsic", id: nid(), name: "json.stringify", args: node.arguments.map((a) => exprFrom(a, sf)), span: sp };
      }
      if (calleeText.startsWith("process.env")) {
        return { op: "intrinsic", id: nid(), name: "env.get", args: node.arguments.map((a) => exprFrom(a, sf)), span: sp };
      }
      return {
        op: "call",
        id: nid(),
        callee: exprFrom(node.expression, sf),
        args: node.arguments.map((a) => exprFrom(a, sf)),
        span: sp,
      };
    }
    case ts.SyntaxKind.BinaryExpression:
      return {
        op: "binary_op",
        id: nid(),
        op_sym: node.operatorToken.getText(sf),
        // serde tag uses "op" for variant; use field name "op" for operator via remap below
        left: exprFrom(node.left, sf),
        right: exprFrom(node.right, sf),
        span: sp,
      };
    case ts.SyntaxKind.ObjectLiteralExpression: {
      const fields = [];
      for (const prop of node.properties) {
        if (ts.isPropertyAssignment(prop)) {
          fields.push([prop.name.getText(sf), exprFrom(prop.initializer, sf)]);
        }
      }
      return { op: "construct", id: nid(), type_name: null, fields, span: sp };
    }
    case ts.SyntaxKind.ArrayLiteralExpression:
      return { op: "list", id: nid(), elements: node.elements.map((e) => exprFrom(e, sf)), span: sp };
    case ts.SyntaxKind.AwaitExpression:
      return { op: "await", id: nid(), value: exprFrom(node.expression, sf), span: sp };
    case ts.SyntaxKind.ElementAccessExpression:
      return {
        op: "index",
        id: nid(),
        collection: exprFrom(node.expression, sf),
        index: exprFrom(node.argumentExpression, sf),
        span: sp,
      };
    case ts.SyntaxKind.TemplateExpression:
    case ts.SyntaxKind.NoSubstitutionTemplateLiteral:
      return { op: "constant", id: nid(), value: node.getText(sf).slice(1, -1), span: sp };
    default:
      // Array.filter intent
      if (ts.isCallExpression(node)) break;
      return { op: "unsupported", id: nid(), original: node.getText(sf).slice(0, 200), span: sp };
  }
  return { op: "unsupported", id: nid(), original: node.getText(sf).slice(0, 200), span: sp };
}

function normalizeExpr(e) {
  if (!e || typeof e !== "object") return e;
  if (e.op === "binary_op" && e.op_sym) {
    const { op_sym, ...rest } = e;
    return { ...rest, op: "binary_op", op: undefined, ...{ op: "binary_op" }, operator: op_sym };
  }
  // Fix binary: PUIR expects field `op` as operator string but tag is also `op`.
  // In Rust enum, BinaryOp { op: String } — serde tag is "op" for variant name via rename.
  // Our Rust uses #[serde(tag = "op")] so variant is "binary_op" and field is also "op" for operator.
  if (e.op === "binary_op" && e.operator) {
    return {
      op: "binary_op",
      id: e.id,
      op: undefined,
      // can't have two op — use the Rust field name. Looking at expr.rs:
      // BinaryOp { op: String, left, right } with tag "op" — CONFLICT!
    };
  }
  return e;
}

// Fix: Rust BinaryOp has field `op` colliding with serde tag `op`. Check expr.rs...
// Yes there's a problem: #[serde(tag = "op")] and field `op: String`.
// I need to rename the operator field in Rust to `operator`.

function fixBinary(e) {
  if (!e || typeof e !== "object") return e;
  if (Array.isArray(e)) return e.map(fixBinary);
  const out = { ...e };
  if (out.op === "binary_op" && out.op_sym) {
    out.operator = out.op_sym;
    delete out.op_sym;
  }
  for (const k of Object.keys(out)) {
    if (k !== "op" && out[k] && typeof out[k] === "object") out[k] = fixBinary(out[k]);
  }
  return out;
}

function stmtFrom(node, sf) {
  const sp = spanOf(sf, node);
  if (ts.isVariableStatement(node)) {
    const decls = [];
    for (const d of node.declarationList.declarations) {
      decls.push({
        op: "declare",
        id: nid(),
        name: d.name.getText(sf),
        mutable: (node.declarationList.flags & ts.NodeFlags.Const) === 0,
        value: d.initializer ? fixBinary(exprFrom(d.initializer, sf)) : null,
        span: sp,
      });
    }
    return decls;
  }
  if (ts.isReturnStatement(node)) {
    return [{
      op: "return",
      id: nid(),
      value: node.expression ? fixBinary(exprFrom(node.expression, sf)) : null,
      span: sp,
    }];
  }
  if (ts.isIfStatement(node)) {
    const thenBody = ts.isBlock(node.thenStatement)
      ? node.thenStatement.statements.flatMap((s) => stmtFrom(s, sf))
      : stmtFrom(node.thenStatement, sf);
    const elseBody = node.elseStatement
      ? ts.isBlock(node.elseStatement)
        ? node.elseStatement.statements.flatMap((s) => stmtFrom(s, sf))
        : stmtFrom(node.elseStatement, sf)
      : [];
    return [{
      op: "branch",
      id: nid(),
      condition: fixBinary(exprFrom(node.expression, sf)),
      then_body: thenBody,
      else_body: elseBody,
      span: sp,
    }];
  }
  if (ts.isExpressionStatement(node)) {
    return [{ op: "expr", id: nid(), expr: fixBinary(exprFrom(node.expression, sf)), span: sp }];
  }
  if (ts.isThrowStatement(node)) {
    return [{ op: "throw", id: nid(), value: fixBinary(exprFrom(node.expression, sf)), span: sp }];
  }
  return [{ op: "unsupported", id: nid(), original: node.getText(sf).slice(0, 200), span: sp }];
}

function functionFrom(node, sf, checker, exported) {
  const name = node.name ? node.name.getText(sf) : "anonymous";
  const params = (node.parameters || []).map((p) => ({
    name: p.name.getText(sf),
    ty: mapType(p.type, sf, checker),
    default: p.initializer ? fixBinary(exprFrom(p.initializer, sf)) : null,
  }));
  const body = node.body && ts.isBlock(node.body)
    ? node.body.statements.flatMap((s) => stmtFrom(s, sf))
    : [];
  const async_ = !!node.modifiers?.some((m) => m.kind === ts.SyntaxKind.AsyncKeyword);
  return {
    kind: "function",
    id: nid(),
    name,
    params,
    return_type: mapType(node.type, sf, checker),
    generics: [],
    visibility: exported ? "public" : "private",
    effects: { io: false, async_: async_, throws: false, env: false, fs: false, network: false },
    body,
    doc: null,
    span: spanOf(sf, node),
    async_,
  };
}

function typeFrom(node, sf, checker, exported) {
  const name = node.name.getText(sf);
  const fields = [];
  if (node.members) {
    for (const m of node.members) {
      if (ts.isPropertySignature(m) || ts.isPropertyDeclaration(m)) {
        fields.push({
          name: m.name.getText(sf),
          ty: mapType(m.type, sf, checker),
          doc: null,
        });
      }
    }
  }
  return {
    kind: "type",
    id: nid(),
    name,
    kind_name: ts.isInterfaceDeclaration(node) ? "interface" : "type_alias",
    fields,
    methods: [],
    visibility: exported ? "public" : "private",
    doc: null,
    span: spanOf(sf, node),
  };
}

// Fix TypeDef serde: kind is tag on PuirItem::Type(TypeDef) — TypeDef has field `kind: String`.
// PuirItem::Type(TypeDef) serializes as { "kind": "type", ...fields of TypeDef }.
// TypeDef.kind field will conflict! Rename TypeDef.kind to type_kind in Rust.

function moduleFrom(sf, checker, routes) {
  const items = [];
  const imports = [];
  const exports = [];
  const isExported = (node) =>
    !!node.modifiers?.some((m) => m.kind === ts.SyntaxKind.ExportKeyword) ||
    (ts.isVariableStatement(node) && node.modifiers?.some((m) => m.kind === ts.SyntaxKind.ExportKeyword));

  for (const stmt of sf.statements) {
    if (ts.isImportDeclaration(stmt)) {
      const from = stmt.moduleSpecifier.text;
      const names = [];
      let def = null;
      const clause = stmt.importClause;
      if (clause?.name) def = clause.name.text;
      if (clause?.namedBindings && ts.isNamedImports(clause.namedBindings)) {
        for (const el of clause.namedBindings.elements) names.push(el.name.text);
      }
      imports.push({ from, names, default: def, span: spanOf(sf, stmt) });
      continue;
    }
    if (ts.isFunctionDeclaration(stmt) && stmt.name) {
      const fn = functionFrom(stmt, sf, checker, isExported(stmt));
      items.push({ kind: "function", ...fn, kind: "function" });
      // unwrap: PuirItem::Function expects { kind: "function", ...Function fields }
      // functionFrom already has kind: "function" — but Function struct doesn't have kind.
      // Serialize as { kind: "function", id, name, params, ... }
      const { kind: _k, ...rest } = fn;
      items.pop();
      items.push({ kind: "function", ...rest });
      if (isExported(stmt)) exports.push({ name: stmt.name.getText(sf), as_name: null });
      continue;
    }
    if (ts.isInterfaceDeclaration(stmt)) {
      const t = typeFrom(stmt, sf, checker, isExported(stmt));
      items.push({
        kind: "type",
        id: t.id,
        name: t.name,
        type_kind: t.kind_name,
        fields: t.fields,
        methods: [],
        visibility: t.visibility,
        doc: null,
        span: t.span,
      });
      if (isExported(stmt)) exports.push({ name: t.name, as_name: null });
      continue;
    }
    if (ts.isTypeAliasDeclaration(stmt)) {
      items.push({
        kind: "type",
        id: nid(),
        name: stmt.name.getText(sf),
        type_kind: "type_alias",
        fields: [],
        methods: [],
        visibility: isExported(stmt) ? "public" : "private",
        doc: null,
        span: spanOf(sf, stmt),
      });
      continue;
    }
    // express routes: app.get("/path", handler)
    if (ts.isExpressionStatement(stmt) && ts.isCallExpression(stmt.expression)) {
      const call = stmt.expression;
      if (ts.isPropertyAccessExpression(call.expression)) {
        const obj = call.expression.expression.getText(sf);
        const method = call.expression.name.getText(sf);
        if ((obj === "app" || obj === "router") && ["get", "post", "put", "delete", "patch"].includes(method)) {
          const routePath = call.arguments[0]?.getText(sf)?.replace(/['"]/g, "") || "/";
          const handler = call.arguments[1]?.getText(sf) || "handler";
          routes.push({ method: method.toUpperCase(), path: routePath, handler, file: rel(sf.fileName) });
        }
      }
    }
    // const / export const
    if (ts.isVariableStatement(stmt)) {
      for (const d of stmt.declarationList.declarations) {
        if (isExported(stmt)) exports.push({ name: d.name.getText(sf), as_name: null });
        if (d.initializer && (ts.isArrowFunction(d.initializer) || ts.isFunctionExpression(d.initializer))) {
          if (!isExported(stmt) && !ts.isArrowFunction(d.initializer)) {
            /* still allow local function consts */
          }
          const fnNode = d.initializer;
          const params = (fnNode.parameters || []).map((p) => ({
            name: p.name.getText(sf),
            ty: mapType(p.type, sf, checker),
            default: null,
          }));
          const body = fnNode.body && ts.isBlock(fnNode.body)
            ? fnNode.body.statements.flatMap((s) => stmtFrom(s, sf))
            : fnNode.body
              ? [{ op: "return", id: nid(), value: fixBinary(exprFrom(fnNode.body, sf)), span: spanOf(sf, fnNode) }]
              : [];
          items.push({
            kind: "function",
            id: nid(),
            name: d.name.getText(sf),
            params,
            return_type: { kind: "unknown" },
            generics: [],
            visibility: "public",
            effects: {
              io: false,
              async_: !!fnNode.modifiers?.some((m) => m.kind === ts.SyntaxKind.AsyncKeyword) || fnNode.asteriskToken != null,
              throws: false,
              env: false,
              fs: false,
              network: false,
            },
            body,
            doc: null,
            span: spanOf(sf, d),
            async_: !!fnNode.modifiers?.some((m) => m.kind === ts.SyntaxKind.AsyncKeyword),
          });
        } else if (d.initializer && ts.isObjectLiteralExpression(d.initializer)) {
          items.push({
            kind: "const",
            id: nid(),
            name: d.name.getText(sf),
            ty: { kind: "map", key: { kind: "string" }, value: { kind: "unknown" } },
            value: fixBinary(exprFrom(d.initializer, sf)),
            visibility: isExported(stmt) ? "public" : "private",
            span: spanOf(sf, d),
          });
        }
      }
    }
  }

  const id = rel(sf.fileName).replace(/\.(tsx?|jsx?|mjs)$/, "");
  return {
    id,
    path: rel(sf.fileName),
    imports,
    exports,
    items,
    doc: null,
    origin_language: sf.fileName.endsWith(".ts") || sf.fileName.endsWith(".tsx") ? "typescript" : "javascript",
    metadata: {},
  };
}

// --- main ---
const { deps, name, main } = loadPackage();
const files = walk(root);
const program = ts.createProgram(files, {
  target: ts.ScriptTarget.ES2020,
  module: ts.ModuleKind.ESNext,
  allowJs: true,
  checkJs: false,
  noEmit: true,
  skipLibCheck: true,
  esModuleInterop: true,
});
const checker = program.getTypeChecker();
const routes = [];
const modules = [];
const projectFiles = [];

for (const sf of program.getSourceFiles()) {
  if (sf.isDeclarationFile) continue;
  if (!files.includes(sf.fileName) && !files.some((f) => path.resolve(f) === path.resolve(sf.fileName))) {
    // only project files
    const r = rel(sf.fileName);
    if (r.startsWith("..")) continue;
  }
  if (!sf.fileName.startsWith(root) && !files.map((f) => path.resolve(f)).includes(path.resolve(sf.fileName))) {
    continue;
  }
  modules.push(moduleFrom(sf, checker, routes));
}

for (const f of files) {
  const r = rel(f);
  const isTest = /test|spec/.test(r);
  projectFiles.push({
    path: r,
    role: isTest ? "test" : "source",
    language: r.endsWith(".ts") || r.endsWith(".tsx") ? "typescript" : "javascript",
    bytes: fs.statSync(f).size,
  });
}
if (fs.existsSync(path.join(root, "package.json"))) {
  projectFiles.push({ path: "package.json", role: "config", language: null, bytes: fs.statSync(path.join(root, "package.json")).size });
}

const hasExpress = deps.some((d) => d.name === "express");
const graph = {
  version: 1,
  name,
  files: projectFiles,
  packages: deps,
  entrypoints: main ? [{ path: main, kind: "bin" }] : files.filter((f) => /index\.(ts|js)$/.test(f)).map((f) => ({ path: rel(f), kind: "bin" })),
  nodes: {},
  edges: [],
  build_system: "npm",
  test_framework: deps.some((d) => d.name === "vitest")
    ? "vitest"
    : deps.some((d) => d.name === "jest")
      ? "jest"
      : deps.some((d) => d.name === "node:test") || true
        ? "node:test"
        : null,
};

const payload = {
  graph,
  modules,
  framework: hasExpress ? "express" : routes.length ? "http" : null,
  database: deps.some((d) => /prisma|pg|postgres|mongoose|sequelize/.test(d.name))
    ? deps.find((d) => /prisma|pg|postgres|mongoose|sequelize/.test(d.name)).name
    : null,
};

// Attach routes metadata to index module if any
if (routes.length) {
  for (const m of modules) {
    if (m.path.includes("route") || m.path.includes("index")) {
      m.metadata.routes = routes;
    }
  }
  payload.modules = modules;
}

process.stdout.write(JSON.stringify(payload));
