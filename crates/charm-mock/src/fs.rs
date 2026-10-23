//! In-memory filesystem.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::lock;

/// Operation names accepted by [`MemFS::fail_on`].
/// Operation name for reads.
pub const OP_READ: &str = "read";
/// Operation name for writes.
pub const OP_WRITE: &str = "write";
/// Operation name for removals.
pub const OP_REMOVE: &str = "remove";
/// Operation name for stats.
pub const OP_STAT: &str = "stat";
/// Operation name for directory listings.
pub const OP_READDIR: &str = "readdir";

const MODE_DIR: u32 = 1 << 31;

fn fs_epoch() -> SystemTime {
    // 2024-01-01 00:00:00 UTC
    std::time::UNIX_EPOCH + Duration::from_secs(1_704_067_200)
}

struct MemFile {
    data: Vec<u8>,
    perm: u32,
    mod_time: SystemTime,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FailKey {
    op: String,
    path: String,
}

struct MemFSInner {
    files: BTreeMap<String, MemFile>,
    dirs: BTreeSet<String>,
    fails: BTreeMap<FailKey, std::sync::Arc<io::Error>>,
    seq: u64,
}

/// Metadata of a file or directory in [`MemFS`].
#[derive(Clone, Debug)]
pub struct FileInfo {
    name: String,
    size: i64,
    mode: u32,
    mod_time: SystemTime,
    is_dir: bool,
}

impl FileInfo {
    /// Base name of the entry. The root is named `"."`.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Size in bytes.
    pub fn size(&self) -> i64 {
        self.size
    }
    /// Full mode bits.
    pub fn mode(&self) -> u32 {
        self.mode
    }
    /// Permission bits (lower 9 bits).
    pub fn perm(&self) -> u32 {
        self.mode & 0o777
    }
    /// Modification time.
    pub fn mod_time(&self) -> SystemTime {
        self.mod_time
    }
    /// Whether the entry is a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    /// System-specific metadata; always `None` here.
    pub fn sys(&self) -> Option<()> {
        None
    }
}

/// Directory entry in [`MemFS`].
#[derive(Clone, Debug)]
pub struct DirEntry {
    name: String,
    is_dir: bool,
    info: FileInfo,
}

impl DirEntry {
    /// Name of the entry.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Whether the entry is a directory.
    pub fn is_dir(&self) -> bool {
        self.is_dir
    }
    /// Type bits: `MODE_DIR` for directories, `0` for files.
    pub fn type_(&self) -> u32 {
        if self.is_dir {
            MODE_DIR
        } else {
            0
        }
    }
    /// Metadata of the entry.
    pub fn info(&self) -> io::Result<FileInfo> {
        Ok(self.info.clone())
    }
}

/// Minimal filesystem interface.
pub trait FileSystem {
    /// Reads the contents of `name`.
    fn read_file(&self, name: &str) -> io::Result<Vec<u8>>;
    /// Writes `data` to `name` with permission `perm`.
    fn write_file(&self, name: &str, data: &[u8], perm: u32) -> io::Result<()>;
    /// Removes `name`.
    fn remove(&self, name: &str) -> io::Result<()>;
    /// Stats `name`.
    fn stat(&self, name: &str) -> io::Result<FileInfo>;
    /// Lists the immediate entries of directory `name`.
    fn read_dir(&self, name: &str) -> io::Result<Vec<DirEntry>>;
}

/// In-memory filesystem, isolated per instance.
#[derive(Clone)]
pub struct MemFS {
    inner: Arc<Mutex<MemFSInner>>,
}

impl std::fmt::Debug for MemFS {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemFS").finish_non_exhaustive()
    }
}

impl Default for MemFS {
    fn default() -> Self {
        Self::new()
    }
}

impl MemFS {
    /// Creates an empty filesystem with only the root directory.
    pub fn new() -> Self {
        let mut dirs = BTreeSet::new();
        dirs.insert(".".to_string());
        Self {
            inner: Arc::new(Mutex::new(MemFSInner {
                files: BTreeMap::new(),
                dirs,
                fails: BTreeMap::new(),
                seq: 0,
            })),
        }
    }

    fn normalize_path(name: &str) -> String {
        if name.is_empty() {
            return ".".to_string();
        }
        let p = name.replace('\\', "/");
        let mut parts: Vec<&str> = Vec::new();
        for seg in p.split('/') {
            match seg {
                "" | "." => {}
                ".." => {
                    if let Some(last) = parts.last() {
                        if *last != ".." {
                            parts.pop();
                            continue;
                        }
                    }
                    parts.push(seg);
                }
                _ => parts.push(seg),
            }
        }
        if parts.is_empty() {
            ".".to_string()
        } else {
            parts.join("/")
        }
    }

    fn normalize_op(op: &str) -> String {
        match op.trim().to_lowercase().as_str() {
            "read" | "readfile" | "open" => OP_READ.to_string(),
            "write" | "writefile" | "create" => OP_WRITE.to_string(),
            "remove" | "delete" => OP_REMOVE.to_string(),
            "stat" => OP_STAT.to_string(),
            "readdir" | "list" => OP_READDIR.to_string(),
            other => other.to_string(),
        }
    }

    fn injected(&self, op: &str, p: &str) -> Option<io::Error> {
        let inner = lock(&self.inner);
        inner
            .fails
            .get(&FailKey {
                op: op.to_string(),
                path: p.to_string(),
            })
            .map(|e| io::Error::new(e.kind(), e.to_string()))
    }

    /// Seeds `name` with `data`, creating parent directories. Returns self.
    #[must_use]
    pub fn seed(&self, name: &str, data: &[u8]) -> Self {
        let p = Self::normalize_path(name);
        if p == "." {
            return self.clone();
        }
        let mut inner = lock(&self.inner);
        if inner.dirs.contains(&p) {
            return self.clone();
        }
        // mkdir parents
        // manual parents:

        let mut cur = p.clone();
        let mut parents = Vec::new();
        loop {
            let parent = match cur.rfind('/') {
                Some(i) => cur[..i].to_string(),
                None => ".".to_string(),
            };
            if parent == cur {
                break;
            }
            parents.push(parent.clone());
            if parent == "." {
                break;
            }
            cur = parent;
        }
        for parent in parents.into_iter().rev() {
            if !inner.dirs.contains(&parent) && !inner.files.contains_key(&parent) {
                inner.dirs.insert(parent);
            }
        }
        // if an ancestor is a file, mkdirParents would fail; check
        // We'll do a simpler check: walk from root
        // (already handled by dirs insert; if file exists, skip seeding)
        let mut ok = true;
        let mut check = p.clone();
        loop {
            let parent = match check.rfind('/') {
                Some(i) => check[..i].to_string(),
                None => ".".to_string(),
            };
            if parent == check {
                break;
            }
            if inner.files.contains_key(&parent) {
                ok = false;
                break;
            }
            if parent == "." {
                break;
            }
            check = parent;
        }
        if !ok {
            return self.clone();
        }

        inner.seq += 1;
        let seq = inner.seq;
        inner.files.insert(
            p,
            MemFile {
                data: data.to_vec(),
                perm: 0o644,
                mod_time: fs_epoch() + Duration::from_secs(seq),
            },
        );
        // ensure parent dirs recorded
        let mut cur2 = Self::normalize_path(name);
        loop {
            let parent = match cur2.rfind('/') {
                Some(i) => cur2[..i].to_string(),
                None => ".".to_string(),
            };
            if parent == cur2 {
                break;
            }
            if parent != "." {
                inner.dirs.insert(parent.clone());
            }
            if parent == "." {
                break;
            }
            cur2 = parent;
        }
        self.clone()
    }

    /// Returns a deep copy of all files, keyed by normalized path.
    pub fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        let inner = lock(&self.inner);
        let mut out = BTreeMap::new();
        for (p, f) in &inner.files {
            out.insert(p.clone(), f.data.clone());
        }
        out
    }

    /// Injects `err` into every subsequent call of operation `op` on `path`.
    /// Pass `None` to use a generic injected error.
    #[must_use]
    pub fn fail_on(&self, op: &str, name: &str, err: Option<io::Error>) -> Self {
        let p = Self::normalize_path(name);
        let err = err.unwrap_or_else(|| io::Error::other("charm-mock: falha injetada"));
        let key = FailKey {
            op: Self::normalize_op(op),
            path: p,
        };
        let mut inner = lock(&self.inner);
        inner.fails.insert(key, std::sync::Arc::new(err));
        self.clone()
    }

    /// Reads `name`.
    pub fn read_file(&self, name: &str) -> io::Result<Vec<u8>> {
        let p = Self::normalize_path(name);
        let op_norm = Self::normalize_op(OP_READ);
        if let Some(err) = self.injected(&op_norm, &p) {
            return Err(err);
        }
        let inner = lock(&self.inner);
        if let Some(f) = inner.files.get(&p) {
            return Ok(f.data.clone());
        }
        if inner.dirs.contains(&p) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "é um diretório",
            ));
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "arquivo não encontrado",
        ))
    }

    /// Writes `data` to `name`.
    pub fn write_file(&self, name: &str, data: &[u8], perm: u32) -> io::Result<()> {
        let p = Self::normalize_path(name);
        if p == "." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "é um diretório",
            ));
        }
        let op_norm = Self::normalize_op(OP_WRITE);
        if let Some(err) = self.injected(&op_norm, &p) {
            return Err(err);
        }
        let mut inner = lock(&self.inner);
        // check injected again under lock not needed
        if inner.dirs.contains(&p) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "é um diretório",
            ));
        }
        // mkdir parents
        let mut cur = p.clone();
        let mut parents = Vec::new();
        loop {
            let parent = match cur.rfind('/') {
                Some(i) => cur[..i].to_string(),
                None => ".".to_string(),
            };
            if parent == cur {
                break;
            }
            parents.push(parent.clone());
            if parent == "." {
                break;
            }
            cur = parent;
        }
        for parent in parents.into_iter().rev() {
            if inner.files.contains_key(&parent) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "não é um diretório",
                ));
            }
            inner.dirs.insert(parent);
        }
        let perm = if perm == 0 { 0o644 } else { perm };
        inner.seq += 1;
        let seq = inner.seq;
        inner.files.insert(
            p,
            MemFile {
                data: data.to_vec(),
                perm: perm & 0o777,
                mod_time: fs_epoch() + Duration::from_secs(seq),
            },
        );
        Ok(())
    }

    /// Removes `name`.
    pub fn remove(&self, name: &str) -> io::Result<()> {
        let p = Self::normalize_path(name);
        let op_norm = Self::normalize_op(OP_REMOVE);
        if let Some(err) = self.injected(&op_norm, &p) {
            return Err(err);
        }
        let mut inner = lock(&self.inner);
        if p == "." {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "não é possível remover a raiz",
            ));
        }
        if inner.files.remove(&p).is_some() {
            return Ok(());
        }
        if inner.dirs.contains(&p) {
            // check children
            let prefix = format!("{p}/");
            let mut has_child = false;
            for key in inner.files.keys() {
                if key.starts_with(&prefix) {
                    has_child = true;
                    break;
                }
            }
            if !has_child {
                for key in &inner.dirs {
                    if key != &p && key.starts_with(&prefix) {
                        has_child = true;
                        break;
                    }
                }
            }
            if has_child {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "diretório não está vazio",
                ));
            }
            inner.dirs.remove(&p);
            return Ok(());
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "arquivo não encontrado",
        ))
    }

    /// Stats `name`.
    pub fn stat(&self, name: &str) -> io::Result<FileInfo> {
        let p = Self::normalize_path(name);
        let op_norm = Self::normalize_op(OP_STAT);
        if let Some(err) = self.injected(&op_norm, &p) {
            return Err(err);
        }
        let inner = lock(&self.inner);
        if let Some(f) = inner.files.get(&p) {
            let base = p.rsplit('/').next().unwrap_or(&p);
            let base = if p == "." { "." } else { base };
            return Ok(FileInfo {
                name: base.to_string(),
                size: i64::try_from(f.data.len()).unwrap_or(i64::MAX),
                mode: f.perm,
                mod_time: f.mod_time,
                is_dir: false,
            });
        }
        if inner.dirs.contains(&p) {
            let base = if p == "." {
                ".".to_string()
            } else {
                p.rsplit('/').next().unwrap_or(&p).to_string()
            };
            return Ok(FileInfo {
                name: base,
                size: 0,
                mode: MODE_DIR | 0o755,
                mod_time: fs_epoch(),
                is_dir: true,
            });
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "arquivo não encontrado",
        ))
    }

    /// Lists directory `name`.
    #[allow(clippy::too_many_lines)]
    pub fn read_dir(&self, name: &str) -> io::Result<Vec<DirEntry>> {
        let p = Self::normalize_path(name);
        let op_norm = Self::normalize_op(OP_READDIR);
        if let Some(err) = self.injected(&op_norm, &p) {
            return Err(err);
        }
        let inner = lock(&self.inner);
        if inner.files.contains_key(&p) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "não é um diretório",
            ));
        }
        if !inner.dirs.contains(&p) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "arquivo não encontrado",
            ));
        }
        let prefix = if p == "." {
            String::new()
        } else {
            format!("{p}/")
        };
        let mut entries_map: BTreeMap<String, DirEntry> = BTreeMap::new();
        for (fpath, f) in &inner.files {
            if let Some(rest) = fpath.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    let child = rest.to_string();
                    entries_map.insert(
                        child.clone(),
                        DirEntry {
                            name: child.clone(),
                            is_dir: false,
                            info: FileInfo {
                                name: child,
                                size: i64::try_from(f.data.len()).unwrap_or(i64::MAX),
                                mode: f.perm,
                                mod_time: f.mod_time,
                                is_dir: false,
                            },
                        },
                    );
                }
            } else if prefix.is_empty() && !fpath.contains('/') {
                // root level file
                let child = fpath.clone();
                entries_map.insert(
                    child.clone(),
                    DirEntry {
                        name: child.clone(),
                        is_dir: false,
                        info: FileInfo {
                            name: child,
                            size: i64::try_from(f.data.len()).unwrap_or(i64::MAX),
                            mode: f.perm,
                            mod_time: f.mod_time,
                            is_dir: false,
                        },
                    },
                );
            }
        }
        for dpath in &inner.dirs {
            if dpath == "." {
                continue;
            }
            if let Some(rest) = dpath.strip_prefix(&prefix) {
                if !rest.is_empty() && !rest.contains('/') {
                    let child = rest.to_string();
                    if !entries_map.contains_key(&child) {
                        entries_map.insert(
                            child.clone(),
                            DirEntry {
                                name: child.clone(),
                                is_dir: true,
                                info: FileInfo {
                                    name: child,
                                    size: 0,
                                    mode: MODE_DIR | 0o755,
                                    mod_time: fs_epoch(),
                                    is_dir: true,
                                },
                            },
                        );
                    }
                }
            } else if prefix.is_empty() && !dpath.contains('/') {
                let child = dpath.clone();
                if !entries_map.contains_key(&child) {
                    entries_map.insert(
                        child.clone(),
                        DirEntry {
                            name: child.clone(),
                            is_dir: true,
                            info: FileInfo {
                                name: child,
                                size: 0,
                                mode: MODE_DIR | 0o755,
                                mod_time: fs_epoch(),
                                is_dir: true,
                            },
                        },
                    );
                }
            }
        }
        // BTreeMap already sorted
        Ok(entries_map.into_values().collect())
    }
}

impl FileSystem for MemFS {
    fn read_file(&self, name: &str) -> io::Result<Vec<u8>> {
        self.read_file(name)
    }
    fn write_file(&self, name: &str, data: &[u8], perm: u32) -> io::Result<()> {
        self.write_file(name, data, perm)
    }
    fn remove(&self, name: &str) -> io::Result<()> {
        self.remove(name)
    }
    fn stat(&self, name: &str) -> io::Result<FileInfo> {
        self.stat(name)
    }
    fn read_dir(&self, name: &str) -> io::Result<Vec<DirEntry>> {
        self.read_dir(name)
    }
}
