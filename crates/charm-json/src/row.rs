use crate::{Kind, Node};

/// Linha visível e formatada da árvore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// Nó representado.
    pub node: Node,
    /// Profundidade (raiz = 0).
    pub depth: usize,
    /// Marcador de expansão.
    pub prefix: String,
    /// Texto formatado.
    pub display: String,
    pub(crate) key_text: String,
    pub(crate) val_text: String,
}

pub(crate) fn flatten(root: &Node, filter: &str, max_depth: usize) -> Vec<Row> {
    let query = filter.to_lowercase();
    fn walk(
        node: &Node,
        depth: usize,
        label: &str,
        query: &str,
        max_depth: usize,
        out: &mut Vec<Row>,
    ) -> bool {
        let matches = query.is_empty()
            || node.path.to_lowercase().contains(query)
            || node.key.to_lowercase().contains(query)
            || node.value.to_lowercase().contains(query);
        let mut visible = Vec::new();
        for (i, c) in node.children.iter().enumerate() {
            let mut probe = Vec::new();
            if walk(c, depth + 1, "", query, max_depth, &mut probe) {
                visible.push((i, c.clone()));
            }
        }
        let relevant = matches || !visible.is_empty();
        if !relevant {
            return false;
        }
        let truncated = max_depth > 0 && depth >= max_depth && !visible.is_empty();
        let expanded = !visible.is_empty() && !truncated && (!query.is_empty() || !node.collapsed);
        let prefix = if visible.is_empty() || truncated {
            "  "
        } else if expanded {
            "▾ "
        } else {
            "▸ "
        }
        .to_string();
        let value = if truncated {
            "…".to_string()
        } else {
            match node.kind {
                Kind::Object => {
                    if node.children.is_empty() {
                        "{}".into()
                    } else {
                        format!("{{{}}}", node.children.len())
                    }
                }
                Kind::Array => {
                    if node.children.is_empty() {
                        "[]".into()
                    } else {
                        format!("[{}]", node.children.len())
                    }
                }
                Kind::String => format!("{:?}", node.value),
                Kind::Number | Kind::Bool | Kind::Null => node.value.clone(),
            }
        };
        let display = if label.is_empty() {
            value.clone()
        } else {
            format!("{label}: {value}")
        };
        out.push(Row {
            node: node.clone(),
            depth,
            prefix,
            display,
            key_text: label.into(),
            val_text: value,
        });
        if expanded {
            for (index, child) in visible {
                let key = if node.kind == Kind::Array {
                    index.to_string()
                } else {
                    format_key(&child.key)
                };
                walk(&child, depth + 1, &key, query, max_depth, out);
            }
        }
        true
    }
    let mut rows = Vec::new();
    walk(root, 0, "", &query, max_depth, &mut rows);
    rows
}
fn format_key(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || "_.-".contains(c)) {
        s.into()
    } else {
        format!("{s:?}")
    }
}
