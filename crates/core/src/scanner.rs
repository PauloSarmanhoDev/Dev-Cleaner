//! Cálculo paralelo de tamanhos (rayon + walkdir).

use crate::model::Target;
use crate::Warnings;
use rayon::prelude::*;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Estado reportado ao callback de progresso.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ScanProgress {
    pub done: usize,
    pub total: usize,
    /// Bytes encontrados até o momento.
    pub bytes: u64,
}

fn warn(w: &Warnings, msg: String) {
    w.lock().unwrap().push(msg);
}

/// Soma (bytes, arquivos) de um caminho; erros viram avisos.
fn measure(path: &Path, w: &Warnings, total_bytes: &AtomicU64) -> (u64, u64) {
    let (mut bytes, mut files) = (0u64, 0u64);
    for entry in WalkDir::new(path).follow_links(false) {
        match entry {
            Ok(e) if e.file_type().is_file() => match e.metadata() {
                Ok(m) => {
                    bytes += m.len();
                    files += 1;
                    total_bytes.fetch_add(m.len(), Ordering::Relaxed);
                }
                Err(err) => warn(w, format!("Sem acesso a {}: {err}", e.path().display())),
            },
            Ok(_) => {}
            Err(err) => warn(w, format!("Ignorado durante varredura: {err}")),
        }
    }
    (bytes, files)
}

/// Filhos imediatos de `dir`, para distribuir o trabalho entre threads.
fn children(dir: &Path, w: &Warnings) -> Vec<std::path::PathBuf> {
    match fs::read_dir(dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(err) => {
            warn(w, format!("Não foi possível listar {}: {err}", dir.display()));
            Vec::new()
        }
    }
}

/// Calcula tamanhos em paralelo; remove alvos vazios e ordena do maior para o menor.
/// `on_progress` é chamado (de várias threads) a cada alvo concluído.
pub fn scan<F>(targets: Vec<Target>, w: &Warnings, on_progress: F) -> Vec<Target>
where
    F: Fn(ScanProgress) + Sync,
{
    let total_items = targets.len();
    let total = AtomicU64::new(0);
    let done = AtomicUsize::new(0);

    let mut result: Vec<Target> = targets
        .into_par_iter()
        .map(|mut t| {
            let (b, f) = children(&t.path, w)
                .par_iter()
                .map(|c| {
                    if c.is_dir() && !c.is_symlink() {
                        measure(c, w, &total)
                    } else {
                        match fs::symlink_metadata(c) {
                            Ok(m) => {
                                total.fetch_add(m.len(), Ordering::Relaxed);
                                (m.len(), 1)
                            }
                            Err(e) => {
                                warn(w, format!("Sem acesso a {}: {e}", c.display()));
                                (0, 0)
                            }
                        }
                    }
                })
                .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
            t.size = b;
            t.files = f;
            on_progress(ScanProgress {
                done: done.fetch_add(1, Ordering::Relaxed) + 1,
                total: total_items,
                bytes: total.load(Ordering::Relaxed),
            });
            t
        })
        .collect();

    result.retain(|t| t.size > 0);
    result.sort_by(|a, b| b.size.cmp(&a.size));
    result
}
