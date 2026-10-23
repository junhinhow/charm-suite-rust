//! Fuzzy subsequence matching.

use std::collections::HashMap;

/// Score constants (mirror the Go original).
const PONTUACAO_BASE: i32 = 10;
const PONTUACAO_INICIO: i32 = 40;
const PONTUACAO_INICIO_PALAVRA: i32 = 25;
const PONTUACAO_CONSECUTIVA: i32 = 15;
const PONTUACAO_PALAVRA_EXATA: i32 = 60;
const PENALIDADE_POR_LACUNA: i32 = 5;

/// A matched item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index of the item in the original input.
    pub index: usize,
    /// Original text of the item.
    pub value: String,
    /// Match score (higher is better).
    pub score: i32,
    /// Rune indices that matched, in the original text.
    pub positions: Vec<usize>,
}

/// Result of [`match_string`].
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Match score.
    pub score: i32,
    /// Rune indices that matched.
    pub positions: Vec<usize>,
    /// Whether the pattern matched.
    pub ok: bool,
}

/// Accent folding table (Latin-1 + Latin Extended-A).
fn acentos() -> &'static HashMap<char, char> {
    use std::sync::OnceLock;
    static M: OnceLock<HashMap<char, char>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        let pairs: &[(char, char)] = &[
            ('á', 'a'),
            ('à', 'a'),
            ('â', 'a'),
            ('ã', 'a'),
            ('ä', 'a'),
            ('å', 'a'),
            ('a', 'a'),
            ('ā', 'a'),
            ('ă', 'a'),
            ('ą', 'a'),
            ('ç', 'c'),
            ('ć', 'c'),
            ('č', 'c'),
            ('ĉ', 'c'),
            ('ċ', 'c'),
            ('ď', 'd'),
            ('đ', 'd'),
            ('é', 'e'),
            ('è', 'e'),
            ('ê', 'e'),
            ('ë', 'e'),
            ('ē', 'e'),
            ('ĕ', 'e'),
            ('ę', 'e'),
            ('ė', 'e'),
            ('ě', 'e'),
            ('ğ', 'g'),
            ('ĝ', 'g'),
            ('ġ', 'g'),
            ('ģ', 'g'),
            ('ĥ', 'h'),
            ('ħ', 'h'),
            ('í', 'i'),
            ('ì', 'i'),
            ('î', 'i'),
            ('ï', 'i'),
            ('ī', 'i'),
            ('ĭ', 'i'),
            ('į', 'i'),
            ('ı', 'i'),
            ('ỉ', 'i'),
            ('ĵ', 'j'),
            ('ķ', 'k'),
            ('ĺ', 'l'),
            ('ļ', 'l'),
            ('ľ', 'l'),
            ('ŀ', 'l'),
            ('ñ', 'n'),
            ('ń', 'n'),
            ('ň', 'n'),
            ('ņ', 'n'),
            ('ó', 'o'),
            ('ò', 'o'),
            ('ô', 'o'),
            ('õ', 'o'),
            ('ö', 'o'),
            ('ø', 'o'),
            ('ō', 'o'),
            ('ŏ', 'o'),
            ('ő', 'o'),
            ('œ', 'o'),
            ('ŕ', 'r'),
            ('ř', 'r'),
            ('ŗ', 'r'),
            ('ś', 's'),
            ('š', 's'),
            ('ŝ', 's'),
            ('ş', 's'),
            ('ť', 't'),
            ('ţ', 't'),
            ('ŧ', 't'),
            ('ú', 'u'),
            ('ù', 'u'),
            ('û', 'u'),
            ('ü', 'u'),
            ('ū', 'u'),
            ('ŭ', 'u'),
            ('ů', 'u'),
            ('ű', 'u'),
            ('ų', 'u'),
            ('ư', 'u'),
            ('ŵ', 'w'),
            ('ý', 'y'),
            ('ÿ', 'y'),
            ('ŷ', 'y'),
            ('ź', 'z'),
            ('ž', 'z'),
            ('ż', 'z'),
            ('æ', 'a'),
        ];
        for (k, v) in pairs {
            m.insert(*k, *v);
        }
        m
    })
}

/// Folds a rune to lowercase + accentless. Returns (folded, keep).
fn dobrar_rune(r: char) -> (char, bool) {
    let r = r.to_lowercase().next().unwrap_or(r);
    if is_mark(r) {
        return ('\0', false);
    }
    if let Some(base) = acentos().get(&r) {
        return (*base, true);
    }
    (r, true)
}

fn is_mark(r: char) -> bool {
    let cp = r as u32;
    // Mn ranges (subset covering the common ones)
    (0x300..=0x36F).contains(&cp) || (0x483..=0x487).contains(&cp) || (0x591..=0x5BD).contains(&cp)
}

/// Folded text with origin mapping.
struct TextoDobrado {
    runes: Vec<char>,
    orig: Vec<usize>,
    src: Vec<char>,
}

fn dobrar_texto(s: &str) -> TextoDobrado {
    let src: Vec<char> = s.chars().collect();
    let mut runes = Vec::with_capacity(src.len());
    let mut orig = Vec::with_capacity(src.len());
    for (i, r) in src.iter().enumerate() {
        let (dr, keep) = dobrar_rune(*r);
        if !keep {
            continue;
        }
        runes.push(dr);
        orig.push(i);
    }
    TextoDobrado { runes, orig, src }
}

fn eh_separador(r: char) -> bool {
    matches!(r, ' ' | '-' | '_' | '.' | '/' | '\\')
}

fn inicio_de_palavra(t: &TextoDobrado, fi: usize) -> bool {
    if fi == 0 || fi >= t.orig.len() {
        return false;
    }
    let atual = t.orig[fi];
    if atual == 0 {
        return true;
    }
    let mut anterior = atual as isize - 1;
    while anterior >= 0 && is_mark(t.src[anterior as usize]) {
        anterior -= 1;
    }
    if anterior < 0 {
        return true;
    }
    if eh_separador(t.src[anterior as usize]) {
        return true;
    }
    t.src[anterior as usize].is_lowercase() && t.src[atual].is_uppercase()
}

fn palavra_exata(t: &TextoDobrado, posicoes: &[usize]) -> bool {
    if posicoes.is_empty() {
        return false;
    }
    for i in 1..posicoes.len() {
        if posicoes[i] != posicoes[i - 1] + 1 {
            return false;
        }
    }
    let inicio = posicoes[0];
    let fim = posicoes[posicoes.len() - 1];
    if inicio > 0 && !inicio_de_palavra(t, inicio) {
        return false;
    }
    if fim + 1 < t.runes.len() {
        let proxima = t.orig[fim + 1];
        if !eh_separador(t.src[proxima]) && !inicio_de_palavra(t, fim + 1) {
            return false;
        }
    }
    true
}

/// Matches `pattern` as a subsequence of `target`, ignoring case and accents.
/// Returns (score, positions, ok).
pub fn match_string(pattern: &str, target: &str) -> (i32, Vec<usize>, bool) {
    let padrao = dobrar_texto(pattern);
    if padrao.runes.is_empty() {
        return (0, Vec::new(), true);
    }
    let texto = dobrar_texto(target);
    if texto.runes.len() < padrao.runes.len() {
        return (0, Vec::new(), false);
    }

    let mut casadas: Vec<usize> = Vec::new();
    let mut pi = 0usize;
    for ti in 0..texto.runes.len() {
        if pi >= padrao.runes.len() {
            break;
        }
        if texto.runes[ti] == padrao.runes[pi] {
            casadas.push(ti);
            pi += 1;
        }
    }
    if pi < padrao.runes.len() {
        return (0, Vec::new(), false);
    }

    let mut score = PONTUACAO_BASE * (padrao.runes.len() as i32);
    let primeira = casadas[0];
    if primeira == 0 {
        score += PONTUACAO_INICIO;
    } else if inicio_de_palavra(&texto, primeira) {
        score += PONTUACAO_INICIO_PALAVRA;
    }
    for i in 1..casadas.len() {
        let atual = casadas[i];
        let anterior = casadas[i - 1];
        if inicio_de_palavra(&texto, atual) {
            score += PONTUACAO_INICIO_PALAVRA;
        }
        if atual == anterior + 1 {
            score += PONTUACAO_CONSECUTIVA;
        } else {
            score -= (atual as i32 - anterior as i32 - 1) * PENALIDADE_POR_LACUNA;
        }
    }
    if palavra_exata(&texto, &casadas) {
        score += PONTUACAO_PALAVRA_EXATA;
    }

    let mut positions = Vec::with_capacity(casadas.len());
    for fi in &casadas {
        positions.push(texto.orig[*fi]);
    }
    (score, positions, true)
}

/// Highlights the matched positions in `value` using `style`.
pub fn highlight(value: &str, positions: &[usize], style: lipgloss::Style) -> String {
    let runes: Vec<char> = value.chars().collect();
    let mut out = String::new();
    let mut i = 0usize;
    while i < runes.len() {
        if positions.contains(&i) {
            // collect consecutive matched
            let mut j = i;
            while j + 1 < runes.len() && positions.contains(&(j + 1)) {
                j += 1;
            }
            let seg: String = runes[i..=j].iter().collect();
            out.push_str(&style.render(&seg));
            i = j + 1;
        } else {
            out.push(runes[i]);
            i += 1;
        }
    }
    out
}
