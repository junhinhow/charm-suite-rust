//! Atualização automática de CLIs: descoberta de releases GitHub, comparação
//! semver, checksum SHA-256 e substituição segura do executável.
//!
//! Este crate porta [`github.com/junhinhow/charm-update`](https://github.com/junhinhow/charm-update).
//! O checksum protege contra corrupção/transporte adulterado, mas não substitui
//! a validação de uma assinatura autenticada do release.
//!
//! ```no_run
//! use charm_update::{Asset, Release, Updater};
//! let release = Release::default();
//! let asset = Asset { name: "app".into(), url: "https://example.com/app".into(), ..Default::default() };
//! Updater::new().apply(&release, &asset)?;
//! # Ok::<(), charm_update::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// Limite padrão de download e conteúdo extraído (200 MiB).
pub const DEFAULT_MAX_SIZE: u64 = 200 << 20;
const MAX_RELEASE_BODY: u64 = 4 << 20;

/// Erros da biblioteca.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Nenhum asset corresponde à plataforma solicitada.
    #[error("update: nenhum asset compatível: {0}")]
    NoAsset(String),
    /// O SHA-256 obtido não corresponde ao esperado.
    #[error("update: checksum divergente: esperado {expected}, obtido {actual}")]
    ChecksumMismatch {
        /// Checksum esperado.
        expected: String,
        /// Checksum calculado a partir dos bytes recebidos.
        actual: String,
    },
    /// O conteúdo excedeu o limite configurado.
    #[error("update: download ou conteúdo extraído excede o tamanho máximo de {0} bytes")]
    TooLarge(u64),
    /// URL insegura, inválida ou redirecionamento recusado.
    #[error("update: URL recusada: {0}")]
    Url(String),
    /// Resposta da API ou archive inválidos.
    #[error("update: formato inválido: {0}")]
    Invalid(String),
    /// Operação HTTP falhou.
    #[error("update: HTTP: {0}")]
    Http(#[from] reqwest::Error),
    /// Operação de filesystem falhou.
    #[error("update: filesystem: {0}")]
    Io(#[from] io::Error),
    /// Entrada de versão não segue o subconjunto semver suportado.
    #[error("update: versão inválida: {0}")]
    Version(String),
}

/// Tipo de resultado da biblioteca.
pub type Result<T> = std::result::Result<T, Error>;

/// Release publicado.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Release {
    /// Tag original, por exemplo `v1.2.3`.
    pub tag: String,
    /// Versão semver parseada.
    pub version: Version,
    /// Notas em Markdown.
    pub notes: String,
    /// Assets na ordem fornecida pela API.
    pub assets: Vec<Asset>,
}

/// Arquivo anexado a um release.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Asset {
    /// Nome do arquivo.
    #[serde(default)]
    pub name: String,
    /// URL de download.
    #[serde(default, rename = "browser_download_url")]
    pub url: String,
    /// Tamanho declarado em bytes.
    #[serde(default)]
    pub size: u64,
    /// SHA-256 hexadecimal opcional.
    #[serde(default, rename = "digest", deserialize_with = "digest_field")]
    pub sha256: String,
}

fn digest_field<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    let s = Option::<String>::deserialize(d)?.unwrap_or_default();
    Ok(normalize_digest(&s).unwrap_or_default())
}
fn normalize_digest(s: &str) -> Option<String> {
    let s = s.trim().strip_prefix("sha256:").unwrap_or(s.trim());
    (s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())).then(|| s.to_ascii_lowercase())
}

/// Versão semver reduzida (build metadata é descartado).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Version {
    /// Componente major.
    pub major: u64,
    /// Componente minor.
    pub minor: u64,
    /// Componente patch.
    pub patch: u64,
    /// Identificadores prerelease sem hífen.
    pub pre: String,
}

impl Version {
    /// Analisa versão semver `MAJOR.MINOR.PATCH`, aceitando prefixo v/V.
    pub fn parse(s: &str) -> Result<Self> {
        let original = s;
        let mut s = s.trim();
        if s.starts_with(['v', 'V']) {
            s = &s[1..];
        }
        let core_meta = s.split_once('+');
        let (core, build) = core_meta.map_or((s, None), |(a, b)| (a, Some(b)));
        if let Some(b) = build {
            validate_ids(b, true).map_err(|e| Error::Version(format!("build metadata: {e}")))?;
        }
        let (core, pre) = core.split_once('-').map_or((core, ""), |(a, b)| (a, b));
        if !pre.is_empty() {
            validate_ids(pre, false).map_err(|e| Error::Version(format!("prerelease: {e}")))?;
        }
        let parts: Vec<_> = core.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::Version(format!(
                "{original:?}: esperado MAJOR.MINOR.PATCH"
            )));
        }
        let mut nums = [0u64; 3];
        for (i, p) in parts.iter().enumerate() {
            if p.is_empty()
                || !p.bytes().all(|b| b.is_ascii_digit())
                || (p.len() > 1 && p.starts_with('0'))
            {
                return Err(Error::Version(format!(
                    "componente numérico inválido {p:?}"
                )));
            }
            nums[i] = p
                .parse()
                .map_err(|_| Error::Version(format!("componente fora do intervalo: {p}")))?;
        }
        Ok(Self {
            major: nums[0],
            minor: nums[1],
            patch: nums[2],
            pre: pre.into(),
        })
    }
    /// Compara precedência semver.
    pub fn compare(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| compare_pre(&self.pre, &other.pre))
    }
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() {
            write!(f, "-{}", self.pre)?;
        }
        Ok(())
    }
}
fn validate_ids(s: &str, build: bool) -> std::result::Result<(), &'static str> {
    if s.is_empty() {
        return Err("identificadores vazios");
    }
    for id in s.split('.') {
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err("identificador inválido");
        }
        if !build && id.bytes().all(|b| b.is_ascii_digit()) && id.len() > 1 && id.starts_with('0') {
            return Err("zero à esquerda");
        }
    }
    Ok(())
}
fn compare_pre(a: &str, b: &str) -> Ordering {
    match (a.is_empty(), b.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        _ => (),
    }
    for (x, y) in a.split('.').zip(b.split('.')) {
        let xn = x.bytes().all(|b| b.is_ascii_digit());
        let yn = y.bytes().all(|b| b.is_ascii_digit());
        let c = match (xn, yn) {
            (true, true) => x
                .trim_start_matches('0')
                .len()
                .cmp(&y.trim_start_matches('0').len())
                .then_with(|| x.trim_start_matches('0').cmp(y.trim_start_matches('0'))),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => x.cmp(y),
        };
        if c != Ordering::Equal {
            return c;
        }
    }
    a.split('.').count().cmp(&b.split('.').count())
}

/// Seleciona deterministically asset por sistema e arquitetura, aliases e preferência de formato.
pub fn choose_asset(os: &str, arch: &str, assets: &[Asset]) -> Result<Asset> {
    let os = os.trim().to_ascii_lowercase();
    let arch = arch.trim().to_ascii_lowercase();
    if os.is_empty() || arch.is_empty() {
        return Err(Error::NoAsset("plataforma inválida".into()));
    }
    let os_alias: &[&str] = match os.as_str() {
        "darwin" => &["darwin", "macos", "osx", "mac"],
        "windows" => &["windows", "win"],
        _ => &[&os],
    };
    let ar_alias: &[&str] = match arch.as_str() {
        "amd64" => &["amd64", "x86_64", "x64"],
        "arm64" => &["arm64", "aarch64"],
        "386" => &["386", "i386", "x86"],
        "arm" => &["arm", "armv6", "armv7", "armhf"],
        _ => &[&arch],
    };
    let mut candidates: Vec<(i32, String, Asset)> = assets
        .iter()
        .filter_map(|a| {
            let name = if a.name.is_empty() {
                a.url.split('?').next()?.rsplit('/').next()?.to_string()
            } else {
                a.name.clone()
            };
            if is_sidecar(&name)
                || !os_alias.iter().any(|w| has_word(&name, w))
                || !ar_alias.iter().any(|w| has_word(&name, w))
            {
                return None;
            }
            let score = if name.to_ascii_lowercase().ends_with(".zip") {
                if os == "windows" {
                    3
                } else {
                    2
                }
            } else if [".tar.gz", ".tgz", ".tar"]
                .iter()
                .any(|s| name.to_ascii_lowercase().ends_with(s))
            {
                if os == "windows" {
                    2
                } else {
                    3
                }
            } else {
                1
            };
            Some((score, name, a.clone()))
        })
        .collect();
    candidates.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then(a.1.cmp(&b.1))
            .then(a.2.url.cmp(&b.2.url))
    });
    candidates.into_iter().next().map(|x| x.2).ok_or_else(|| {
        Error::NoAsset(format!(
            "para {os}/{arch}; disponíveis: {}",
            assets
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })
}
fn has_word(s: &str, w: &str) -> bool {
    let s = s.to_ascii_lowercase();
    let w = w.to_ascii_lowercase();
    s.match_indices(&w).any(|(i, _)| {
        let e = i + w.len();
        (i == 0
            || s.as_bytes()[i - 1].is_ascii_whitespace()
            || b"._-".contains(&s.as_bytes()[i - 1]))
            && (e == s.len()
                || s.as_bytes()[e].is_ascii_whitespace()
                || b"._-".contains(&s.as_bytes()[e]))
    })
}
fn is_sidecar(n: &str) -> bool {
    [
        ".sha256",
        ".sha256sum",
        ".sha512",
        ".sig",
        ".asc",
        ".minisig",
        ".pem",
        ".txt",
        ".json",
        ".sbom",
        ".spdx",
        ".cyclonedx",
        ".intoto",
        ".att",
        ".bundle",
    ]
    .iter()
    .any(|s| n.to_ascii_lowercase().ends_with(s))
}

/// Configuração imutável do checker e updater.
#[derive(Clone)]
pub struct Options {
    /// Raiz da API GitHub.
    pub base_url: String,
    /// Versão de fallback do checker.
    pub current_version: String,
    /// SHA-256 esperado; vazio usa o digest do asset.
    pub checksum: String,
    /// Caminho/nome de destino; vazio usa o executável atual.
    pub binary_name: PathBuf,
    /// Limite de transferência e descompactação.
    pub max_size: u64,
    /// Timeout HTTP.
    pub timeout: Duration,
    /// Callback de progresso `(bytes recebidos, total conhecido ou zero)`.
    pub progress: Option<Arc<dyn Fn(u64, u64) + Send + Sync>>,
}
impl Options {
    /// Define a raiz HTTP da API (HTTP só é aceito para loopback).
    #[must_use]
    pub fn with_base_url(mut self, value: impl Into<String>) -> Self {
        self.base_url = value.into();
        self
    }
    /// Define a versão atual usada quando `Checker::check` recebe string vazia.
    #[must_use]
    pub fn with_current_version(mut self, value: impl Into<String>) -> Self {
        self.current_version = value.into();
        self
    }
    /// Define o SHA-256 esperado (hexadecimal, opcionalmente prefixado `sha256:`).
    #[must_use]
    pub fn with_checksum(mut self, value: impl Into<String>) -> Self {
        self.checksum = value.into();
        self
    }
    /// Define caminho ou nome do executável de destino.
    #[must_use]
    pub fn with_binary_name(mut self, value: impl Into<PathBuf>) -> Self {
        self.binary_name = value.into();
        self
    }
    /// Define limite de bytes para transferência e membro descompactado.
    #[must_use]
    pub fn with_max_size(mut self, value: u64) -> Self {
        if value > 0 {
            self.max_size = value;
        }
        self
    }
    /// Define timeout de cada operação HTTP.
    #[must_use]
    pub fn with_timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }
    /// Registra callback de progresso do download.
    #[must_use]
    pub fn with_progress(mut self, callback: impl Fn(u64, u64) + Send + Sync + 'static) -> Self {
        self.progress = Some(Arc::new(callback));
        self
    }
}
impl std::fmt::Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options")
            .field("base_url", &self.base_url)
            .field("current_version", &self.current_version)
            .field("checksum", &self.checksum)
            .field("binary_name", &self.binary_name)
            .field("max_size", &self.max_size)
            .field("timeout", &self.timeout)
            .field("progress", &self.progress.as_ref().map(|_| "configured"))
            .finish()
    }
}
impl Default for Options {
    fn default() -> Self {
        Self {
            base_url: "https://api.github.com".into(),
            current_version: String::new(),
            checksum: String::new(),
            binary_name: PathBuf::new(),
            max_size: DEFAULT_MAX_SIZE,
            timeout: Duration::from_secs(300),
            progress: None,
        }
    }
}

/// Consulta a última release estável do GitHub.
#[derive(Debug, Clone)]
pub struct Checker {
    owner: String,
    repo: String,
    options: Options,
    client: Client,
}
impl Checker {
    /// Cria checker para `owner/repo`.
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Result<Self> {
        Self::with_options(owner, repo, Options::default())
    }
    /// Cria checker com opções.
    pub fn with_options(
        owner: impl Into<String>,
        repo: impl Into<String>,
        mut options: Options,
    ) -> Result<Self> {
        if options.max_size == 0 {
            options.max_size = DEFAULT_MAX_SIZE;
        }
        let owner = owner.into();
        let repo = repo.into();
        if owner.trim().is_empty() || repo.trim().is_empty() {
            return Err(Error::Invalid("owner e repo são obrigatórios".into()));
        }
        let client = make_client(&options)?;
        Ok(Self {
            owner,
            repo,
            options,
            client,
        })
    }
    /// Busca o último release estável.
    pub fn latest(&self) -> Result<Release> {
        let mut url = reqwest::Url::parse(self.options.base_url.trim_end_matches('/'))
            .map_err(|e| Error::Url(e.to_string()))?;
        url.path_segments_mut()
            .map_err(|()| Error::Url("base URL não aceita segmentos".into()))?
            .pop_if_empty()
            .push("repos")
            .push(&self.owner)
            .push(&self.repo)
            .push("releases")
            .push("latest");
        let response = self
            .client
            .get(url)
            .header(ACCEPT, "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?
            .error_for_status()?;
        if response
            .content_length()
            .is_some_and(|n| n > MAX_RELEASE_BODY)
        {
            return Err(Error::Invalid("corpo da API excede 4 MiB".into()));
        }
        let mut body = Vec::new();
        response.take(MAX_RELEASE_BODY + 1).read_to_end(&mut body)?;
        if body.len() as u64 > MAX_RELEASE_BODY {
            return Err(Error::Invalid("corpo da API excede 4 MiB".into()));
        }
        let value: ApiRelease = serde_json::from_slice(&body)
            .map_err(|e| Error::Invalid(format!("resposta inválida da API: {e}")))?;
        value.into_release()
    }
    /// Retorna release e se é mais novo que `current` ou a versão configurada.
    pub fn check(&self, current: &str) -> Result<(Release, bool)> {
        let c = if current.trim().is_empty() {
            &self.options.current_version
        } else {
            current
        };
        let cv = Version::parse(c)?;
        let r = self.latest()?;
        let newer = r.version.compare(&cv) == Ordering::Greater;
        Ok((r, newer))
    }
}
#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    assets: Vec<Asset>,
}
impl ApiRelease {
    fn into_release(self) -> Result<Release> {
        let version = Version::parse(&self.tag_name)?;
        Ok(Release {
            tag: self.tag_name,
            version,
            notes: self.body,
            assets: self.assets,
        })
    }
}

fn make_client(o: &Options) -> Result<Client> {
    let u = reqwest::Url::parse(&o.base_url).map_err(|e| Error::Url(e.to_string()))?;
    let loopback = u.host_str().is_some_and(|h| {
        h == "localhost"
            || h.parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if u.scheme() != "https" && !(u.scheme() == "http" && loopback) {
        return Err(Error::Url(
            "base URL deve usar HTTPS (HTTP permitido apenas em loopback)".into(),
        ));
    }
    Ok(Client::builder()
        .timeout(o.timeout)
        .user_agent("charm-update")
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 10 {
                return attempt.error("redirecionamentos demais (limite 10)");
            }
            if attempt.url().scheme() == "https" {
                attempt.follow()
            } else {
                attempt.error("redirecionamento inseguro: HTTPS obrigatório")
            }
        }))
        .build()?)
}

/// Opções e instalação de atualizações.
#[derive(Debug, Clone)]
pub struct Updater {
    options: Options,
    client: Client,
}
impl Updater {
    /// Cria updater com configurações padrão.
    pub fn new() -> Self {
        Self::with_options(Options::default()).expect("configuração padrão válida")
    }
    /// Cria updater com opções explícitas.
    pub fn with_options(mut options: Options) -> Result<Self> {
        if options.max_size == 0 {
            options.max_size = DEFAULT_MAX_SIZE;
        }
        let client = make_client(&options)?;
        if !options.checksum.is_empty() && normalize_digest(&options.checksum).is_none() {
            return Err(Error::Invalid(
                "checksum deve conter 64 dígitos hexadecimais".into(),
            ));
        }
        Ok(Self { options, client })
    }
    /// Instala um asset após download, SHA-256 e extração opcional.
    pub fn apply(&self, _release: &Release, asset: &Asset) -> Result<()> {
        if asset.url.trim().is_empty() {
            return Err(Error::NoAsset(format!("asset {:?} sem URL", asset.name)));
        }
        let url = reqwest::Url::parse(&asset.url).map_err(|e| Error::Url(e.to_string()))?;
        if url.scheme() != "https"
            && !(url.scheme() == "http"
                && self.options.base_url.starts_with("http://")
                && url.host_str().is_some_and(|h| {
                    h == "localhost"
                        || h.parse::<std::net::IpAddr>()
                            .is_ok_and(|ip| ip.is_loopback())
                }))
        {
            return Err(Error::Url("URL do asset exige HTTPS".into()));
        }
        let target = self.target_path()?;
        let parent = target.parent().unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let filename = if asset.name.is_empty() {
            url.path_segments()
                .and_then(Iterator::last)
                .unwrap_or("asset")
                .to_string()
        } else {
            asset.name.clone()
        };
        let expected = if self.options.checksum.is_empty() {
            asset.sha256.clone()
        } else {
            self.options.checksum.clone()
        };
        let expected = if expected.is_empty() {
            String::new()
        } else {
            normalize_digest(&expected).ok_or_else(|| Error::Invalid("checksum inválido".into()))?
        };
        let mut response = self
            .client
            .get(url)
            .header(ACCEPT, "application/octet-stream")
            .send()?
            .error_for_status()?;
        let expected_size = if asset.size > 0 {
            asset.size
        } else {
            response.content_length().unwrap_or(0)
        };
        if let Some(progress) = &self.options.progress {
            progress(0, expected_size);
        }
        let mut temp = tempfile::NamedTempFile::new_in(parent)?;
        let mut hasher = Sha256::new();
        let mut total = 0u64;
        let mut buf = vec![0u8; 32 * 1024].into_boxed_slice();
        loop {
            let n = response.read(&mut buf)?;
            if n == 0 {
                break;
            }
            total = total.saturating_add(n as u64);
            if total > self.options.max_size {
                return Err(Error::TooLarge(self.options.max_size));
            }
            hasher.update(&buf[..n]);
            temp.write_all(&buf[..n])?;
            if let Some(progress) = &self.options.progress {
                progress(total, expected_size);
            }
        }
        temp.as_file().sync_all()?;
        if let Some(progress) = &self.options.progress {
            progress(total, expected_size);
        }
        let actual = format!("{:x}", hasher.finalize());
        if !expected.is_empty() && expected != actual {
            return Err(Error::ChecksumMismatch { expected, actual });
        }
        let is_archive = [".tar.gz", ".tgz", ".tar", ".zip"]
            .iter()
            .any(|s| filename.to_ascii_lowercase().ends_with(s));
        let staged = if is_archive {
            let bytes = fs::read(temp.path())?;
            let extracted = extract(
                &bytes,
                &filename,
                target.file_name().and_then(|s| s.to_str()).unwrap_or(""),
                self.options.max_size,
            )?;
            let mut f = tempfile::NamedTempFile::new_in(parent)?;
            f.write_all(&extracted)?;
            f.as_file().sync_all()?;
            f
        } else {
            temp
        };
        install(staged, target)
    }
    fn target_path(&self) -> Result<PathBuf> {
        if !self.options.binary_name.as_os_str().is_empty() {
            let p = &self.options.binary_name;
            if p.is_absolute() || p.components().count() > 1 {
                return Ok(p.clone());
            }
            let exe = std::env::current_exe()?;
            return Ok(exe.parent().unwrap_or(Path::new(".")).join(p));
        }
        Ok(std::env::current_exe()?)
    }
}
impl Default for Updater {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::too_many_lines)]
fn extract(data: &[u8], name: &str, want: &str, max: u64) -> Result<Vec<u8>> {
    let zipped = name.to_ascii_lowercase().ends_with(".zip");
    let members: Vec<String> = if zipped {
        let mut z = zip::ZipArchive::new(io::Cursor::new(data))
            .map_err(|e| Error::Invalid(format!("zip inválido: {e}")))?;
        let mut names = Vec::new();
        for i in 0..z.len() {
            let f = z.by_index(i).map_err(|e| Error::Invalid(e.to_string()))?;
            if !f.is_dir()
                && f.unix_mode()
                    .is_none_or(|m| m & 0o170_000 == 0 || m & 0o170_000 == 0o100_000)
                && safe_member(f.name())
            {
                names.push(f.name().to_string());
            }
        }
        names
    } else {
        let reader: Box<dyn Read> = if name.to_ascii_lowercase().ends_with(".gz")
            || name.to_ascii_lowercase().ends_with(".tgz")
        {
            Box::new(GzDecoder::new(io::Cursor::new(data)))
        } else {
            Box::new(io::Cursor::new(data))
        };
        let mut archive = tar::Archive::new(reader);
        let mut names = Vec::new();
        for entry in archive
            .entries()
            .map_err(|e| Error::Invalid(format!("tar inválido: {e}")))?
        {
            let e = entry.map_err(|e| Error::Invalid(e.to_string()))?;
            if !e.header().entry_type().is_file() {
                continue;
            }
            let p = e
                .path()
                .map_err(|e| Error::Invalid(e.to_string()))?
                .to_string_lossy()
                .into_owned();
            if safe_member(&p) {
                names.push(p);
            }
        }
        names
    };
    let matches: Vec<_> = members
        .iter()
        .filter(|p| {
            Path::new(p)
                .file_name()
                .is_some_and(|b| b.eq_ignore_ascii_case(std::ffi::OsStr::new(want)))
        })
        .collect();
    if matches.len() > 1 {
        return Err(Error::Invalid(format!(
            "mais de um arquivo corresponde a {want:?}: {}",
            matches
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let selected = matches.first().map(|p| (*p).clone()).or_else(|| {
        let candidates: Vec<_> = members.iter().filter(|p| !is_doc(p)).collect();
        (candidates.len() == 1).then(|| candidates[0].clone())
    });
    let Some(selected) = selected else {
        let candidates = members
            .iter()
            .filter(|p| !is_doc(p))
            .map(String::as_str)
            .collect::<Vec<_>>();
        return Err(Error::Invalid(format!(
            "arquivo ambíguo/ausente no archive; candidatos: {}",
            candidates.join(", ")
        )));
    };
    if zipped {
        let mut z = zip::ZipArchive::new(io::Cursor::new(data))
            .map_err(|e| Error::Invalid(format!("zip inválido: {e}")))?;
        let f = z
            .by_name(&selected)
            .map_err(|e| Error::Invalid(e.to_string()))?;
        let mut out = Vec::new();
        f.take(max.saturating_add(1)).read_to_end(&mut out)?;
        if out.len() as u64 > max {
            return Err(Error::TooLarge(max));
        }
        Ok(out)
    } else {
        let reader: Box<dyn Read> = if name.to_ascii_lowercase().ends_with(".gz")
            || name.to_ascii_lowercase().ends_with(".tgz")
        {
            Box::new(GzDecoder::new(io::Cursor::new(data)))
        } else {
            Box::new(io::Cursor::new(data))
        };
        for entry in tar::Archive::new(reader)
            .entries()
            .map_err(|e| Error::Invalid(format!("tar inválido: {e}")))?
        {
            let entry = entry.map_err(|e| Error::Invalid(e.to_string()))?;
            if entry.path().is_ok_and(|p| p.to_string_lossy() == selected) {
                let mut out = Vec::new();
                entry.take(max.saturating_add(1)).read_to_end(&mut out)?;
                if out.len() as u64 > max {
                    return Err(Error::TooLarge(max));
                }
                return Ok(out);
            }
        }
        Err(Error::Invalid(format!(
            "entrada {selected:?} ausente no archive"
        )))
    }
}
fn safe_member(n: &str) -> bool {
    let normalized = n.replace('\\', "/");
    if normalized.starts_with('/') || normalized.as_bytes().get(1) == Some(&b':') {
        return false;
    }
    let p = Path::new(&normalized);
    !p.is_absolute() && p.components().all(|c| matches!(c, Component::Normal(_)))
}
fn is_doc(n: &str) -> bool {
    let b = Path::new(n)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    [
        "license",
        "licence",
        "readme",
        "changelog",
        "changes",
        "copying",
        "notice",
        "authors",
        "contributors",
    ]
    .iter()
    .any(|p| b.starts_with(p))
        || [".md", ".txt", ".rst", ".html", ".1", ".5"]
            .iter()
            .any(|s| b.ends_with(s))
}

#[cfg(unix)]
#[allow(clippy::needless_pass_by_value)]
fn install(mut staged: tempfile::NamedTempFile, target: PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    staged
        .as_file()
        .set_permissions(fs::Permissions::from_mode(0o755))?;
    staged.as_file().sync_all()?;
    staged.persist(&target).map_err(|e| Error::Io(e.error))?;
    if let Some(p) = target.parent() {
        if let Ok(d) = fs::File::open(p) {
            let _ = d.sync_all();
        }
    }
    Ok(())
}
#[cfg(windows)]
#[allow(clippy::needless_pass_by_value)]
fn install(staged: tempfile::NamedTempFile, target: PathBuf) -> Result<()> {
    let mut old_name = target.as_os_str().to_os_string();
    old_name.push(".old");
    let mut old = PathBuf::from(old_name);
    if old.exists() && fs::remove_file(&old).is_err() {
        let mut fallback = old.as_os_str().to_os_string();
        fallback.push(format!(".{}", std::process::id()));
        old = PathBuf::from(fallback);
        let _ = fs::remove_file(&old);
    }
    if target.exists() {
        fs::rename(&target, &old)?;
    }
    if let Err(e) = staged.persist(&target) {
        if old.exists() {
            let _ = fs::rename(&old, &target);
        }
        return Err(Error::Io(e.error));
    }
    let _ = fs::remove_file(old);
    Ok(())
}
#[cfg(not(any(unix, windows)))]
#[allow(clippy::needless_pass_by_value)]
fn install(staged: tempfile::NamedTempFile, target: PathBuf) -> Result<()> {
    staged.persist(target).map_err(|e| Error::Io(e.error))?;
    Ok(())
}

/// Informações do pacote `charm-update`.
pub mod info {
    /// Versão atual do crate.
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn semver_precedence_and_validation() {
        assert_eq!(
            Version::parse("v1.2.3-rc.10+meta").unwrap().to_string(),
            "1.2.3-rc.10"
        );
        assert_eq!(
            Version::parse("1.2.3-rc.2")
                .unwrap()
                .compare(&Version::parse("1.2.3-rc.10").unwrap()),
            Ordering::Less
        );
        assert!(Version::parse("01.2.3").is_err());
    }
    #[test]
    fn platform_asset_prefers_native_archive() {
        let a = vec![
            Asset {
                name: "app_linux_amd64.zip".into(),
                ..Default::default()
            },
            Asset {
                name: "app_linux_amd64.tar.gz".into(),
                ..Default::default()
            },
            Asset {
                name: "app_linux_amd64.tar.gz.sha256".into(),
                ..Default::default()
            },
        ];
        assert_eq!(
            choose_asset("linux", "amd64", &a).unwrap().name,
            "app_linux_amd64.tar.gz"
        );
    }
    #[test]
    fn archive_path_traversal_filtered_and_ambiguous_fails() {
        assert!(!safe_member("../../bin/app"));
        assert!(safe_member("bin/app"));
        assert!(extract(b"bad", "update.zip", "app", 100).is_err());
    }
    #[test]
    fn checksum_normalization() {
        assert_eq!(
            normalize_digest(&format!("sha256:{}", "A".repeat(64))).unwrap(),
            "a".repeat(64)
        );
    }

    #[test]
    fn github_payload_deserializes_digest() {
        let digest = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let payload = format!(
            r#"{{"tag_name":"v2.1.0","body":"notes","assets":[{{"name":"app_linux_amd64","browser_download_url":"https://example.invalid/app","size":3,"digest":"sha256:{}"}}]}}"#,
            digest.to_uppercase()
        );
        let parsed: ApiRelease = serde_json::from_str(&payload).unwrap();
        let release = parsed.into_release().unwrap();
        assert_eq!(release.version.to_string(), "2.1.0");
        assert_eq!(release.assets[0].sha256, digest);
    }

    #[test]
    fn tar_extraction_selects_requested_regular_file() {
        let mut bytes = Vec::new();
        {
            let mut tar = tar::Builder::new(&mut bytes);
            for (name, body) in [
                ("README.md", b"help".as_slice()),
                ("bundle/app", b"binary".as_slice()),
            ] {
                let mut h = tar::Header::new_gnu();
                h.set_size(body.len() as u64);
                h.set_mode(0o755);
                h.set_cksum();
                tar.append_data(&mut h, name, body).unwrap();
            }
            tar.finish().unwrap();
        }
        assert_eq!(
            extract(&bytes, "bundle.tar", "app", 1024).unwrap(),
            b"binary"
        );
    }

    #[test]
    fn zip_extracts_only_selected_member_and_rejects_duplicate_names() {
        use std::io::Write as _;
        let mut bytes = io::Cursor::new(Vec::new());
        {
            let mut archive = zip::ZipWriter::new(&mut bytes);
            let options = zip::write::SimpleFileOptions::default();
            archive.start_file("release/app", options).unwrap();
            archive.write_all(b"binary").unwrap();
            archive.start_file("release/README.md", options).unwrap();
            archive.write_all(&[b'x'; 128]).unwrap();
            archive.finish().unwrap();
        }
        assert_eq!(
            extract(bytes.get_ref(), "release.zip", "app", 16).unwrap(),
            b"binary"
        );

        let mut duplicate = io::Cursor::new(Vec::new());
        {
            let mut archive = zip::ZipWriter::new(&mut duplicate);
            let options = zip::write::SimpleFileOptions::default();
            for path in ["one/app", "two/app"] {
                archive.start_file(path, options).unwrap();
                archive.write_all(b"binary").unwrap();
            }
            archive.finish().unwrap();
        }
        assert!(extract(duplicate.get_ref(), "release.zip", "app", 16).is_err());
    }
}
