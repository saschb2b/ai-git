use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    Python,
    Rust,
    Go,
    Unknown,
}

pub fn detect_language(file_path: &str) -> Language {
    match file_path.rsplit('.').next() {
        Some("ts" | "tsx") => Language::TypeScript,
        Some("py") => Language::Python,
        Some("rs") => Language::Rust,
        Some("go") => Language::Go,
        _ => Language::Unknown,
    }
}

pub fn get_parser(lang: Language) -> Result<tree_sitter::Parser> {
    let mut parser = tree_sitter::Parser::new();
    let language = match lang {
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::Go => tree_sitter_go::LANGUAGE.into(),
        Language::Unknown => bail!("cannot create parser for unknown language"),
    };
    parser.set_language(&language)?;
    Ok(parser)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticChange {
    pub change_type: String,
    pub symbol_name: String,
    pub file_path: String,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub kind: String,
    pub name: String,
    pub text: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// Returns the list of tree-sitter node kinds that represent top-level
/// definitions for a given language, along with a human-readable semantic kind
/// string for each.
fn definition_node_kinds(lang: Language) -> &'static [(&'static str, &'static str)] {
    match lang {
        Language::TypeScript => &[
            ("function_declaration", "function"),
            ("class_declaration", "class"),
            ("interface_declaration", "interface"),
            ("type_alias_declaration", "type alias"),
            ("method_definition", "method"),
            ("lexical_declaration", "declaration"),
        ],
        Language::Python => &[
            ("function_definition", "function"),
            ("class_definition", "class"),
        ],
        Language::Rust => &[
            ("function_item", "function"),
            ("struct_item", "struct"),
            ("enum_item", "enum"),
            ("impl_item", "impl"),
            ("type_item", "type alias"),
            ("trait_item", "trait"),
        ],
        Language::Go => &[
            ("function_declaration", "function"),
            ("method_declaration", "method"),
            ("type_declaration", "type"),
            ("type_spec", "type"),
        ],
        Language::Unknown => &[],
    }
}

/// Try to extract the name of a definition node.
///
/// First attempts `child_by_field_name("name")`, then falls back to scanning
/// immediate children for the first `identifier` or `type_identifier` node.
fn extract_node_name(node: &tree_sitter::Node, source: &[u8]) -> Option<String> {
    // Try the "name" field first.
    if let Some(name_node) = node.child_by_field_name("name") {
        if let Ok(text) = name_node.utf8_text(source) {
            return Some(text.to_string());
        }
    }

    // Fallback: first child whose kind is identifier or type_identifier.
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Some(child) = node.child(i) {
            let kind = child.kind();
            if kind == "identifier" || kind == "type_identifier" {
                if let Ok(text) = child.utf8_text(source) {
                    return Some(text.to_string());
                }
            }
        }
    }

    None
}

/// Walk the root node's children (and, for Go's `type_declaration`, their
/// children) to find top-level definition nodes and extract [`Definition`]s.
pub fn extract_definitions(source: &str, lang: Language) -> Result<Vec<Definition>> {
    if lang == Language::Unknown {
        bail!("cannot extract definitions for unknown language");
    }

    let mut parser = get_parser(lang)?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("tree-sitter failed to parse source"))?;

    let root = tree.root_node();
    let source_bytes = source.as_bytes();
    let kinds = definition_node_kinds(lang);

    // Build a quick lookup: node_kind -> semantic_kind
    let kind_map: HashMap<&str, &str> = kinds.iter().copied().collect();

    let mut definitions = Vec::new();

    collect_definitions(root, source_bytes, &kind_map, lang, &mut definitions);

    Ok(definitions)
}

/// Recursively (but shallowly) collect definitions starting from `node`.
///
/// For most languages we only look at direct children of the root.  For Go we
/// also descend into `type_declaration` to find the inner `type_spec` nodes.
fn collect_definitions(
    node: tree_sitter::Node,
    source: &[u8],
    kind_map: &HashMap<&str, &str>,
    lang: Language,
    out: &mut Vec<Definition>,
) {
    let child_count = node.child_count();
    for i in 0..child_count {
        let child = match node.child(i) {
            Some(c) => c,
            None => continue,
        };

        let node_kind = child.kind();

        // Go: `type_declaration` wraps one or more `type_spec` children.
        if lang == Language::Go && node_kind == "type_declaration" {
            collect_definitions(child, source, kind_map, lang, out);
            continue;
        }

        if let Some(&semantic_kind) = kind_map.get(node_kind) {
            if let Some(name) = extract_node_name(&child, source) {
                let text = child
                    .utf8_text(source)
                    .unwrap_or("")
                    .to_string();
                out.push(Definition {
                    kind: semantic_kind.to_string(),
                    name,
                    text,
                    start_line: child.start_position().row,
                    end_line: child.end_position().row,
                });
            }
        }
    }
}

/// Produce a set of [`SemanticChange`] entries by comparing two versions of
/// source code at the tree-sitter CST level.
///
/// Returns an error if `lang` is [`Language::Unknown`].
pub fn semantic_diff(
    old_source: &str,
    new_source: &str,
    lang: Language,
) -> Result<Vec<SemanticChange>> {
    if lang == Language::Unknown {
        bail!("cannot perform semantic diff for unknown language");
    }

    let old_defs = extract_definitions(old_source, lang)?;
    let new_defs = extract_definitions(new_source, lang)?;

    // Index by name for quick lookup.  When there are duplicate names (e.g.
    // overloaded methods – rare in the languages we handle) we just keep the
    // last one; this is a best-effort heuristic.
    let old_map: HashMap<&str, &Definition> =
        old_defs.iter().map(|d| (d.name.as_str(), d)).collect();
    let new_map: HashMap<&str, &Definition> =
        new_defs.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut changes = Vec::new();

    // Detect added & modified definitions.
    for def in &new_defs {
        match old_map.get(def.name.as_str()) {
            None => {
                changes.push(SemanticChange {
                    change_type: "added".to_string(),
                    symbol_name: def.name.clone(),
                    file_path: String::new(),
                    details: format!("added {} `{}`", def.kind, def.name),
                });
            }
            Some(old_def) => {
                if old_def.text != def.text {
                    changes.push(SemanticChange {
                        change_type: "modified".to_string(),
                        symbol_name: def.name.clone(),
                        file_path: String::new(),
                        details: format!("modified {} `{}`", def.kind, def.name),
                    });
                }
                // Same text → no change, skip.
            }
        }
    }

    // Detect removed definitions.
    for def in &old_defs {
        if !new_map.contains_key(def.name.as_str()) {
            changes.push(SemanticChange {
                change_type: "removed".to_string(),
                symbol_name: def.name.clone(),
                file_path: String::new(),
                details: format!("removed {} `{}`", def.kind, def.name),
            });
        }
    }

    Ok(changes)
}
