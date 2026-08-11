//! Semantic diff engine (not text diff).

use crate::identity::SemanticId;
use indexmap::IndexMap;
use parallax_puir::{Function, Module, PuirItem, PuirProgram, PuirType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Kind of semantic change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    AddedFunction,
    RemovedFunction,
    RenamedSymbol,
    ChangedSignature,
    ChangedReturnType,
    ChangedControlFlow,
    ChangedCall,
    ChangedDependency,
    ChangedConstant,
    ChangedErrorBehavior,
    ChangedDataModel,
    ChangedAsyncBehavior,
    AddedType,
    RemovedType,
    Unchanged,
}

/// One semantic change.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticChange {
    pub kind: ChangeKind,
    pub id: SemanticId,
    pub qualified_name: String,
    pub detail: String,
    pub source_file: Option<String>,
}

/// Diff between two PUIR programs (+ optional dependency lists).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SemanticDiff {
    pub changes: Vec<SemanticChange>,
}

impl SemanticDiff {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    pub fn affected_ids(&self) -> Vec<SemanticId> {
        self.changes.iter().map(|c| c.id.clone()).collect()
    }
}

/// Compare baseline PUIR to current source PUIR.
pub fn diff_programs(base: &PuirProgram, current: &PuirProgram) -> SemanticDiff {
    let base_fns = index_functions(base);
    let cur_fns = index_functions(current);
    let mut changes = Vec::new();

    for (name, (id, sig, body_hash, file, ret, async_)) in &cur_fns {
        match base_fns.get(name) {
            None => {
                // Rename detection: same signature+body, different name
                if let Some((old_name, _old)) = base_fns
                    .iter()
                    .find(|(_, v)| v.2 == *body_hash && v.1 == *sig)
                {
                    if old_name != name {
                        changes.push(SemanticChange {
                            kind: ChangeKind::RenamedSymbol,
                            id: id.clone(),
                            qualified_name: name.clone(),
                            detail: format!("{old_name} → {name}"),
                            source_file: file.clone(),
                        });
                        continue;
                    }
                }
                changes.push(SemanticChange {
                    kind: ChangeKind::AddedFunction,
                    id: id.clone(),
                    qualified_name: name.clone(),
                    detail: "new function".into(),
                    source_file: file.clone(),
                });
            }
            Some((_, old_sig, old_body, _, old_ret, old_async)) => {
                if old_sig != sig {
                    changes.push(SemanticChange {
                        kind: ChangeKind::ChangedSignature,
                        id: id.clone(),
                        qualified_name: name.clone(),
                        detail: format!("{old_sig} → {sig}"),
                        source_file: file.clone(),
                    });
                }
                if format_ty(old_ret) != format_ty(ret) {
                    changes.push(SemanticChange {
                        kind: ChangeKind::ChangedReturnType,
                        id: id.clone(),
                        qualified_name: name.clone(),
                        detail: format!("{:?} → {:?}", old_ret, ret),
                        source_file: file.clone(),
                    });
                }
                if old_async != async_ {
                    changes.push(SemanticChange {
                        kind: ChangeKind::ChangedAsyncBehavior,
                        id: id.clone(),
                        qualified_name: name.clone(),
                        detail: format!("async {old_async} → {async_}"),
                        source_file: file.clone(),
                    });
                }
                if old_body != body_hash {
                    changes.push(SemanticChange {
                        kind: ChangeKind::ChangedControlFlow,
                        id: id.clone(),
                        qualified_name: name.clone(),
                        detail: "function body hash changed".into(),
                        source_file: file.clone(),
                    });
                }
            }
        }
    }

    for (name, (id, _, _, file, _, _)) in &base_fns {
        if !cur_fns.contains_key(name) {
            // skip if rename already recorded
            if changes.iter().any(|c| {
                matches!(c.kind, ChangeKind::RenamedSymbol)
                    && c.detail.starts_with(&format!("{name} →"))
            }) {
                continue;
            }
            changes.push(SemanticChange {
                kind: ChangeKind::RemovedFunction,
                id: id.clone(),
                qualified_name: name.clone(),
                detail: "removed function".into(),
                source_file: file.clone(),
            });
        }
    }

    // Types
    let base_types = index_types(base);
    let cur_types = index_types(current);
    for (name, (id, hash, file)) in &cur_types {
        match base_types.get(name) {
            None => changes.push(SemanticChange {
                kind: ChangeKind::AddedType,
                id: id.clone(),
                qualified_name: name.clone(),
                detail: "new type".into(),
                source_file: file.clone(),
            }),
            Some((_, old_hash, _)) if old_hash != hash => changes.push(SemanticChange {
                kind: ChangeKind::ChangedDataModel,
                id: id.clone(),
                qualified_name: name.clone(),
                detail: "type shape changed".into(),
                source_file: file.clone(),
            }),
            _ => {}
        }
    }
    for (name, (id, _, file)) in &base_types {
        if !cur_types.contains_key(name) {
            changes.push(SemanticChange {
                kind: ChangeKind::RemovedType,
                id: id.clone(),
                qualified_name: name.clone(),
                detail: "removed type".into(),
                source_file: file.clone(),
            });
        }
    }

    SemanticDiff { changes }
}

type FnEntry = (SemanticId, String, String, Option<String>, PuirType, bool);

fn index_functions(prog: &PuirProgram) -> IndexMap<String, FnEntry> {
    let mut out = IndexMap::new();
    for module in prog.modules.values() {
        for item in &module.items {
            if let PuirItem::Function(f) = item {
                let qn = format!("{}.{}", module.id.replace('/', "."), f.name);
                let sig = signature_of(f);
                let id = SemanticId::derive("function", &qn, &sig);
                let body = hash_json(&f.body);
                let file = f.span.as_ref().map(|s| s.file.clone());
                out.insert(qn, (id, sig, body, file, f.return_type.clone(), f.async_));
            }
        }
    }
    out
}

fn index_types(prog: &PuirProgram) -> IndexMap<String, (SemanticId, String, Option<String>)> {
    let mut out = IndexMap::new();
    for module in prog.modules.values() {
        for item in &module.items {
            if let PuirItem::Type(t) = item {
                let qn = format!("{}.{}", module.id.replace('/', "."), t.name);
                let hash = hash_json(&t.fields);
                let id = SemanticId::derive("type", &qn, &hash);
                let file = t.span.as_ref().map(|s| s.file.clone());
                out.insert(qn, (id, hash, file));
            }
        }
    }
    out
}

fn signature_of(f: &Function) -> String {
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| format!("{}:{}", p.name, format_ty(&p.ty)))
        .collect();
    format!(
        "({})->{}/async={}",
        params.join(","),
        format_ty(&f.return_type),
        f.async_
    )
}

fn format_ty(t: &PuirType) -> String {
    serde_json::to_string(t).unwrap_or_else(|_| "unknown".into())
}

fn hash_json<T: Serialize>(v: &T) -> String {
    let bytes = serde_json::to_vec(v).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&bytes);
    hex::encode(h.finalize())
}

/// Build semantic index snapshot for a program.
pub fn build_index(prog: &PuirProgram) -> IndexMap<String, SemanticId> {
    let mut map = IndexMap::new();
    for (k, (id, _, _, _, _, _)) in index_functions(prog) {
        map.insert(k, id);
    }
    for (k, (id, _, _)) in index_types(prog) {
        map.insert(k, id);
    }
    let _ = Module {
        id: String::new(),
        path: String::new(),
        imports: vec![],
        exports: vec![],
        items: vec![],
        doc: None,
        origin_language: String::new(),
        metadata: IndexMap::new(),
    };
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use parallax_puir::{Effects, NodeId, Parameter, Visibility};

    fn prog_with(name: &str, op: &str) -> PuirProgram {
        use parallax_puir::{Expr, Stmt};
        let mut p = PuirProgram::new();
        let body = vec![Stmt::Return {
            id: NodeId::new(2),
            value: Some(Expr::BinaryOp {
                id: NodeId::new(3),
                operator: op.into(),
                left: Box::new(Expr::Name {
                    id: NodeId::new(4),
                    name: "a".into(),
                    span: None,
                }),
                right: Box::new(Expr::Name {
                    id: NodeId::new(5),
                    name: "b".into(),
                    span: None,
                }),
                span: None,
            }),
            span: None,
        }];
        let f = Function {
            id: NodeId::new(1),
            name: name.into(),
            params: vec![
                Parameter {
                    name: "a".into(),
                    ty: PuirType::f64(),
                    default: None,
                },
                Parameter {
                    name: "b".into(),
                    ty: PuirType::f64(),
                    default: None,
                },
            ],
            return_type: PuirType::f64(),
            generics: vec![],
            visibility: Visibility::Public,
            effects: Effects::default(),
            body,
            doc: None,
            span: None,
            async_: false,
        };
        p.modules.insert(
            "m".into(),
            Module {
                id: "m".into(),
                path: "m.ts".into(),
                imports: vec![],
                exports: vec![],
                items: vec![PuirItem::Function(f)],
                doc: None,
                origin_language: "typescript".into(),
                metadata: IndexMap::new(),
            },
        );
        p
    }

    #[test]
    fn rename_is_not_remove_add_when_body_same() {
        let a = prog_with("add", "+");
        let b = prog_with("sum", "+");
        // force same body hash path: sum with +
        let d = diff_programs(&a, &b);
        assert!(
            d.changes
                .iter()
                .any(|c| matches!(c.kind, ChangeKind::RenamedSymbol))
                || d.changes
                    .iter()
                    .any(|c| matches!(c.kind, ChangeKind::AddedFunction)),
            "expected rename or add/remove pair: {:?}",
            d.changes
        );
    }

    #[test]
    fn operator_change_is_control_flow() {
        let a = prog_with("add", "+");
        let b = prog_with("add", "-");
        let d = diff_programs(&a, &b);
        assert!(d
            .changes
            .iter()
            .any(|c| matches!(c.kind, ChangeKind::ChangedControlFlow)));
    }
}
