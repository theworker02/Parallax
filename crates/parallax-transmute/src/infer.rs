//! Dynamic → static type inference from PUIR evidence.

use indexmap::IndexMap;
use parallax_puir::{
    Confidence, Expr, PuirItem, PuirProgram, PuirType, Stmt, TypeEvidence, TypeInferenceBundle,
    TypeInferenceReport,
};

/// Infer types for names appearing in the program.
pub fn infer_types(program: &PuirProgram) -> TypeInferenceBundle {
    let mut evidence: IndexMap<String, Vec<TypeEvidence>> = IndexMap::new();
    for module in program.modules.values() {
        for item in &module.items {
            collect_item(item, &mut evidence);
        }
    }
    let mut reports = IndexMap::new();
    for (symbol, ev) in evidence {
        let (ty, conf, alts, review) = conclude(&ev);
        reports.insert(
            symbol.clone(),
            TypeInferenceReport {
                symbol,
                inferred: ty,
                confidence: conf,
                evidence: ev,
                alternatives: alts,
                manual_review: review,
            },
        );
    }
    TypeInferenceBundle { reports }
}

fn collect_item(item: &PuirItem, out: &mut IndexMap<String, Vec<TypeEvidence>>) {
    match item {
        PuirItem::Function(f) => {
            for p in &f.params {
                if !matches!(p.ty, PuirType::Unknown) {
                    push(
                        out,
                        &p.name,
                        "parameter_annotation",
                        format!("{:?}", p.ty),
                        f.span.as_ref().map(|s| s.file.clone()),
                        f.span.as_ref().map(|s| s.line),
                    );
                }
            }
            for s in &f.body {
                collect_stmt(s, out);
            }
        }
        PuirItem::Const { name, ty, value, span, .. } => {
            if !matches!(ty, PuirType::Unknown) {
                push(
                    out,
                    name,
                    "const_annotation",
                    format!("{ty:?}"),
                    span.as_ref().map(|s| s.file.clone()),
                    span.as_ref().map(|s| s.line),
                );
            }
            collect_expr(value, out);
        }
        PuirItem::Type(t) => {
            for field in &t.fields {
                push(
                    out,
                    &format!("{}.{}", t.name, field.name),
                    "field",
                    format!("{:?}", field.ty),
                    t.span.as_ref().map(|s| s.file.clone()),
                    t.span.as_ref().map(|s| s.line),
                );
            }
        }
        PuirItem::Unsupported { .. } => {}
    }
}

fn collect_stmt(stmt: &Stmt, out: &mut IndexMap<String, Vec<TypeEvidence>>) {
    match stmt {
        Stmt::Declare {
            name,
            value: Some(v),
            span,
            ..
        } => {
            if let Some(t) = literal_type(v) {
                push(
                    out,
                    name,
                    "assigned_literal",
                    format!("{t:?}"),
                    span.as_ref().map(|s| s.file.clone()),
                    span.as_ref().map(|s| s.line),
                );
            }
            collect_expr(v, out);
        }
        Stmt::Declare { .. } => {}
        Stmt::Assign { target, value, span, .. } => {
            if let Some(t) = literal_type(value) {
                push(
                    out,
                    target,
                    "assigned_literal",
                    format!("{t:?}"),
                    span.as_ref().map(|s| s.file.clone()),
                    span.as_ref().map(|s| s.line),
                );
            }
            collect_expr(value, out);
        }
        Stmt::Return { value: Some(v), .. } => collect_expr(v, out),
        Stmt::Branch {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expr(condition, out);
            for s in then_body {
                collect_stmt(s, out);
            }
            for s in else_body {
                collect_stmt(s, out);
            }
        }
        Stmt::Expr { expr, .. } => collect_expr(expr, out),
        _ => {}
    }
}

fn collect_expr(expr: &Expr, out: &mut IndexMap<String, Vec<TypeEvidence>>) {
    match expr {
        Expr::BinaryOp {
            left,
            right,
            operator,
            ..
        } => {
            if matches!(
                operator.as_str(),
                "+" | "-" | "*" | "/" | "%" | "<" | ">" | "<=" | ">="
            ) {
                if let Expr::Name { name, .. } = left.as_ref() {
                    push(out, name, "numeric_operator", operator.clone(), None, None);
                }
                if let Expr::Name { name, .. } = right.as_ref() {
                    push(out, name, "numeric_operator", operator.clone(), None, None);
                }
            }
            collect_expr(left, out);
            collect_expr(right, out);
        }
        Expr::Index { collection, index, .. } => {
            if let Expr::Name { name, .. } = index.as_ref() {
                push(out, name, "used_as_index", "integer-like".into(), None, None);
            }
            collect_expr(collection, out);
            collect_expr(index, out);
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_expr(a, out);
            }
        }
        Expr::Await { value, .. } => collect_expr(value, out),
        Expr::Filter {
            collection,
            predicate,
            ..
        }
        | Expr::Map {
            collection,
            body: predicate,
            ..
        } => {
            collect_expr(collection, out);
            collect_expr(predicate, out);
        }
        _ => {}
    }
}

fn literal_type(expr: &Expr) -> Option<PuirType> {
    match expr {
        Expr::Constant { value, .. } => {
            if value.is_i64() || value.as_u64().is_some() {
                Some(PuirType::i64())
            } else if value.is_f64() {
                Some(PuirType::f64())
            } else if value.is_boolean() {
                Some(PuirType::Bool)
            } else if value.is_string() {
                Some(PuirType::String)
            } else if value.is_null() {
                Some(PuirType::Optional {
                    inner: Box::new(PuirType::Unknown),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn push(
    out: &mut IndexMap<String, Vec<TypeEvidence>>,
    symbol: &str,
    kind: &str,
    detail: String,
    file: Option<String>,
    line: Option<u32>,
) {
    out.entry(symbol.to_string()).or_default().push(TypeEvidence {
        symbol: symbol.to_string(),
        kind: kind.into(),
        detail,
        file,
        line,
    });
}

fn conclude(ev: &[TypeEvidence]) -> (PuirType, Confidence, Vec<PuirType>, bool) {
    let mut votes: IndexMap<String, u32> = IndexMap::new();
    for e in ev {
        let key = if e.detail.contains("Int") || e.kind == "used_as_index" || e.kind == "numeric_operator"
        {
            if e.detail.contains("Float") {
                "float"
            } else {
                "int"
            }
        } else if e.detail.contains("String") || e.detail.contains("string") {
            "string"
        } else if e.detail.contains("Bool") {
            "bool"
        } else {
            "unknown"
        };
        *votes.entry(key.into()).or_default() += 1;
    }
    let best = votes.iter().max_by_key(|(_, c)| *c).map(|(k, _)| k.as_str());
    match best {
        Some("int") => (PuirType::i64(), Confidence::High, vec![PuirType::f64()], false),
        Some("float") => (PuirType::f64(), Confidence::High, vec![PuirType::i64()], false),
        Some("string") => (PuirType::String, Confidence::High, Vec::new(), false),
        Some("bool") => (PuirType::Bool, Confidence::High, Vec::new(), false),
        _ => (
            PuirType::Unknown,
            Confidence::Low,
            vec![PuirType::String, PuirType::Bytes],
            true,
        ),
    }
}
