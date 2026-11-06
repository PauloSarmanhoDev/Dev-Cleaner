//! Cálculo paralelo de tamanhos (rayon + walkdir), com progresso e cancelamento.

use crate::i18n::tf;
use crate::model::{Target, TargetKind};
use crate::{CancelToken, Warnings, procs};
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Instant, UNIX_EPOCH};
use walkdir::WalkDir;

/// Estado reportado ao callback de progresso (a cada ~100 ms, não por arquivo).
#[derive(Debug, Clone, Serialize)]
pub struct ScanProgress {
    pub done: usize,
    pub total: usize,
    /// Bytes encontrados até o momento.
    pub bytes: u64,
    /// Arquivos lidos até o momento.
    pub files: u64,
    /// Arquivo ou pasta sendo lido agora (amostra: com várias threads é um dos vários em curso).
    pub current: String,
}

struct Shared {
    bytes: AtomicU64,
    files: AtomicU64,
    done: AtomicUsize,
    current: Mutex<String>,
    last_emit: AtomicU64,
    start: Instant,
    total: usize,
}

#[derive(Default, Clone, Copy)]
struct Acc {
    bytes: u64,
    files: u64,
    newest: u64,
}

impl Acc {
    fn merge(self, o: Acc) -> Acc {
        Acc {
            bytes: self.bytes + o.bytes,
            files: self.files + o.files,
            newest: self.newest.max(o.newest),
        }
    }
}

fn warn(w: &Warnings, msg: String) {
    w.lock().unwrap().push(msg);
}

fn mtime(m: &fs::Metadata) -> u64 {
    m.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn set_current(s: &Shared, p: &Path) {
    if let Ok(mut c) = s.current.lock() {
        *c = p.display().to_string();
    }
}

type Emit<'a> = &'a (dyn Fn(bool) + Sync);

fn measure(path: &Path, w: &Warnings, s: &Shared, cancel: &CancelToken, emit: Emit) -> Acc {
    let mut acc = Acc::default();
    for entry in WalkDir::new(path).follow_links(false) {
        if cancel.is_cancelled() {
            break;
        }
        match entry {
            Ok(e) if e.file_type().is_file() => match e.metadata() {
                Ok(m) => {
                    acc.bytes += m.len();
                    acc.files += 1;
                    acc.newest = acc.newest.max(mtime(&m));
                    s.bytes.fetch_add(m.len(), Ordering::Relaxed);
                    if s.files.fetch_add(1, Ordering::Relaxed) & 255 == 0 {
                        set_current(s, e.path());
                        emit(false);
                    }
                }
                Err(err) => warn(w, tf("warn.access", &[&e.path().display(), &err])),
            },
            Ok(_) => {}
            Err(err) => warn(w, tf("warn.scan", &[&err])),
        }
    }
    acc
}

/// Filhos imediatos de `dir`, para distribuir o trabalho entre threads.
fn children(dir: &Path, w: &Warnings) -> Vec<std::path::PathBuf> {
    match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(err) => {
            warn(w, tf("warn.list", &[&dir.display(), &err]));
            Vec::new()
        }
    }
}

/// Calcula tamanhos em paralelo; remove alvos vazios, anota IDEs abertas e ordena do maior
/// para o menor. Se cancelado, devolve lista vazia (verifique `cancel.is_cancelled()`).
pub fn scan<F>(
    targets: Vec<Target>,
    w: &Warnings,
    cancel: &CancelToken,
    on_progress: F,
) -> Vec<Target>
where
    F: Fn(ScanProgress) + Sync,
{
    let s = Shared {
        bytes: AtomicU64::new(0),
        files: AtomicU64::new(0),
        done: AtomicUsize::new(0),
        current: Mutex::new(String::new()),
        last_emit: AtomicU64::new(0),
        start: Instant::now(),
        total: targets.len(),
    };
    let report = || ScanProgress {
        done: s.done.load(Ordering::Relaxed),
        total: s.total,
        bytes: s.bytes.load(Ordering::Relaxed),
        files: s.files.load(Ordering::Relaxed),
        current: s.current.lock().map(|c| c.clone()).unwrap_or_default(),
    };
    // Limita a frequência dos eventos (importante para a GUI).
    let emit = |force: bool| {
        let now = s.start.elapsed().as_millis() as u64;
        let last = s.last_emit.load(Ordering::Relaxed);
        if force
            || (now.saturating_sub(last) >= 100
                && s.last_emit
                    .compare_exchange(last, now, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok())
        {
            on_progress(report());
        }
    };

    let mut result: Vec<Target> = targets
        .into_par_iter()
        .map(|mut t| {
            if cancel.is_cancelled() {
                return t;
            }
            // Alvos que não são pasta (Docker) já vêm com o tamanho calculado.
            if t.kind != TargetKind::Dir {
                s.done.fetch_add(1, Ordering::Relaxed);
                emit(true);
                return t;
            }
            set_current(&s, &t.path);
            let acc = children(&t.path, w)
                .par_iter()
                .map(|c| {
                    if c.is_dir() && !c.is_symlink() {
                        measure(c, w, &s, cancel, &emit)
                    } else {
                        match fs::symlink_metadata(c) {
                            Ok(m) => {
                                s.bytes.fetch_add(m.len(), Ordering::Relaxed);
                                s.files.fetch_add(1, Ordering::Relaxed);
                                Acc {
                                    bytes: m.len(),
                                    files: 1,
                                    newest: mtime(&m),
                                }
                            }
                            Err(e) => {
                                warn(w, tf("warn.access", &[&c.display(), &e]));
                                Acc::default()
                            }
                        }
                    }
                })
                .reduce(Acc::default, Acc::merge);
            t.size = acc.bytes;
            t.files = acc.files;
            t.last_used = (acc.newest > 0).then_some(acc.newest);
            s.done.fetch_add(1, Ordering::Relaxed);
            emit(true);
            t
        })
        .collect();

    if cancel.is_cancelled() {
        return Vec::new();
    }
    result.retain(|t| t.size > 0);
    result.sort_by_key(|t| std::cmp::Reverse(t.size));
    procs::annotate(&mut result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    #[test]
    fn measures_size_files_and_drops_empty() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        fs::create_dir_all(a.join("sub")).unwrap();
        fs::write(a.join("f1"), vec![0u8; 1000]).unwrap();
        fs::write(a.join("sub").join("f2"), vec![0u8; 500]).unwrap();
        let empty = dir.path().join("empty");
        fs::create_dir(&empty).unwrap();
        let ts = vec![
            Target::new(Category::Tool, "a", a, false),
            Target::new(Category::Tool, "e", empty, false),
        ];
        let w = Warnings::default();
        let out = scan(ts, &w, &CancelToken::default(), |_| {});
        assert_eq!(out.len(), 1);
        assert_eq!((out[0].size, out[0].files), (1500, 2));
        assert!(out[0].last_used.is_some());
    }

    #[test]
    fn cancel_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("f"), b"x").unwrap();
        let c = CancelToken::default();
        c.cancel();
        let ts = vec![Target::new(
            Category::Tool,
            "a",
            dir.path().to_path_buf(),
            false,
        )];
        assert!(scan(ts, &Warnings::default(), &c, |_| {}).is_empty());
    }
}
