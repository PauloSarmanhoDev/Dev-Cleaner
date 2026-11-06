//! Remoção resiliente: arquivos bloqueados/sem permissão são pulados e registrados.

use crate::model::Target;
use crate::Warnings;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Remove o conteúdo de `path` (contents_first) e depois o diretório, se possível.
/// Retorna bytes liberados.
fn remove_tree(path: &Path, w: &Warnings) -> u64 {
    let mut freed = 0u64;
    for entry in WalkDir::new(path).contents_first(true).follow_links(false) {
        let e = match entry {
            Ok(e) => e,
            Err(err) => {
                w.lock().unwrap().push(format!("Ignorado: {err}"));
                continue;
            }
        };
        let p = e.path();
        let res = if e.file_type().is_dir() {
            fs::remove_dir(p) // falha se ainda houver arquivos bloqueados dentro
        } else {
            let len = e.metadata().map(|m| m.len()).unwrap_or(0);
            let r = fs::remove_file(p);
            if r.is_ok() {
                freed += len;
            }
            r
        };
        if let Err(err) = res {
            // Diretório não vazio é consequência de um aviso anterior; não duplicar.
            if !e.file_type().is_dir() {
                w.lock().unwrap().push(format!("Bloqueado/sem permissão: {} ({err})", p.display()));
            }
        }
    }
    freed
}

/// Remove todos os alvos (em paralelo) e retorna total de bytes liberados.
/// `on_progress(concluídos, total)` é chamado a cada alvo finalizado.
pub fn clean<F>(targets: &[Target], w: &Warnings, on_progress: F) -> u64
where
    F: Fn(usize, usize) + Sync,
{
    let done = AtomicUsize::new(0);
    targets
        .par_iter()
        .map(|t| {
            let freed = remove_tree(&t.path, w);
            on_progress(done.fetch_add(1, Ordering::Relaxed) + 1, targets.len());
            freed
        })
        .sum()
}
