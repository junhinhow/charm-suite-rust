use std::{fmt, num::ParseIntError};

/// Tipo do valor armazenado em um nó.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Objeto JSON / mapeamento YAML.
    Object,
    /// Array JSON / sequência YAML.
    Array,
    /// Valor textual.
    String,
    /// Valor numérico preservado como texto.
    Number,
    /// Valor booleano.
    Bool,
    /// Valor nulo.
    Null,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Object => "object",
            Self::Array => "array",
            Self::String => "string",
            Self::Number => "number",
            Self::Bool => "bool",
            Self::Null => "null",
        })
    }
}

/// Nó de uma árvore JSON/YAML. Valores escalares são strings e caminhos usam RFC 6901.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Chave no pai (vazia na raiz e nos itens de array).
    pub key: String,
    /// Tipo do nó.
    pub kind: Kind,
    /// Valor escalar, ou string vazia para contêineres.
    pub value: String,
    /// Filhos em sua ordem original.
    pub children: Vec<Node>,
    /// JSON Pointer RFC 6901.
    pub path: String,
    /// Se o contêiner está recolhido.
    pub collapsed: bool,
}

impl Node {
    pub(crate) fn is_container(&self) -> bool {
        matches!(self.kind, Kind::Object | Kind::Array)
    }

    /// Procura um nó por JSON Pointer estrito.
    pub fn find(&self, path: &str) -> Option<&Node> {
        if path.is_empty() {
            return Some(self);
        }
        if !path.starts_with('/') {
            return None;
        }
        let mut current = self;
        for raw in path[1..].split('/') {
            let token = decode_token(raw)?;
            current = match current.kind {
                Kind::Object => current.children.iter().find(|child| child.key == token)?,
                Kind::Array => current.children.get(parse_index(&token).ok()?)?,
                _ => return None,
            };
        }
        Some(current)
    }

    /// Recolhe o contêiner no caminho indicado.
    pub fn collapse(&mut self, path: &str) {
        if let Some(node) = self.find_mut(path) {
            if node.is_container() {
                node.collapsed = true;
            }
        }
    }
    /// Expande o contêiner no caminho indicado.
    pub fn expand(&mut self, path: &str) {
        if let Some(node) = self.find_mut(path) {
            if node.is_container() {
                node.collapsed = false;
            }
        }
    }
    /// Alterna o estado de recolhimento do contêiner indicado.
    pub fn toggle(&mut self, path: &str) {
        if let Some(node) = self.find_mut(path) {
            if node.is_container() {
                node.collapsed = !node.collapsed;
            }
        }
    }
    /// Recolhe todos os contêineres, incluindo a raiz.
    pub fn collapse_all(&mut self) {
        if self.is_container() {
            self.collapsed = true;
        }
        for child in &mut self.children {
            child.collapse_all();
        }
    }
    /// Expande todos os contêineres.
    pub fn expand_all(&mut self) {
        if self.is_container() {
            self.collapsed = false;
        }
        for child in &mut self.children {
            child.expand_all();
        }
    }
    pub(crate) fn find_mut(&mut self, path: &str) -> Option<&mut Node> {
        if path.is_empty() {
            return Some(self);
        }
        if !path.starts_with('/') {
            return None;
        }
        let mut parts = path[1..].split('/');
        let token = decode_token(parts.next()?)?;
        let next = match self.kind {
            Kind::Object => self.children.iter_mut().find(|n| n.key == token)?,
            Kind::Array => self.children.get_mut(parse_index(&token).ok()?)?,
            _ => return None,
        };
        let rest = parts.collect::<Vec<_>>();
        if rest.is_empty() {
            Some(next)
        } else {
            next.find_mut(&format!("/{}", rest.join("/")))
        }
    }
    /// Devolve as linhas visíveis em pré-ordem, respeitando o estado de colapso.
    pub fn flatten(&self) -> Vec<crate::Row> {
        crate::row::flatten(self, "", 0)
    }
}

pub(crate) fn escape_token(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
fn decode_token(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '~' {
            match chars.next()? {
                '0' => out.push('~'),
                '1' => out.push('/'),
                _ => return None,
            }
        } else {
            out.push(c);
        }
    }
    Some(out)
}
fn parse_index(s: &str) -> Result<usize, ParseIntError> {
    if s.len() > 1 && s.starts_with('0') {
        return "".parse::<usize>();
    }
    s.parse()
}
