//! Remoção resiliente: lixeira (padrão) ou permanente. Arquivos bloqueados/sem permissão
//! são pulados e registrados. Toda ação é registrada no log de auditoria.

use crate::i18n::tf;
use crate::model::{Target, TargetKind};
use crate::{CancelToken, Warnings, safety};
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeleteMode {
    /// Move para a lixeira do sistema (reversível).
    #[default]
    Trash,
    /// Apaga definitivamente.
    Permanent,
}

#[derive(Debug, Clone)]
pub struct CleanOptions {
    pub mode: DeleteMode,
    /// Máximo de diretórios removidos em paralelo (limita a carga no disco).
    pub threads: usize,
    pub cancel: CancelToken,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            mode: DeleteMode::Trash,
            threads: 4,
            cancel: CancelToken::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CleanReport {
    pub freed: u64,
    pub removed: usize,
    pub skipped: usize,
}

fn warn(w: &Warnings, msg: String) {
    tracing::warn!("{msg}");
    w.lock().unwrap().push(msg);
}

/// Remove um arquivo; se estiver somente leitura, tenta liberar o atributo e repetir.
fn remove_file_force(p: &Path) -> std::io::Result<()> {
    match fs::remove_file(p) {
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            if let Ok(meta) = fs::metadata(p) {
                let mut perm = meta.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perm.set_readonly(false);
                let _ = fs::set_permissions(p, perm);
            }
            fs::remove_file(p)
        }
        r => r,
    }
}

/// Remove o conteúdo de `path` (contents_first) e depois o diretório, se possível.
/// Retorna bytes liberados.
fn remove_tree(path: &Path, w: &Warnings, cancel: &CancelToken) -> u64 {
    let mut freed = 0u64;
    for entry in WalkDir::new(path).contents_first(true).follow_links(false) {
        if cancel.is_cancelled() {
            break;
        }
        let e = match entry {
            Ok(e) => e,
            Err(err) => {
                warn(w, tf("warn.walk", &[&err]));
                continue;
            }
        };
        let p = e.path();
        let is_dir = e.file_type().is_dir();
        let res = if is_dir {
            fs::remove_dir(p) // falha se ainda houver arquivos bloqueados dentro
        } else {
            let len = e.metadata().map(|m| m.len()).unwrap_or(0);
            let r = remove_file_force(p);
            if r.is_ok() {
                freed += len;
            }
            r
        };
        // Diretório não vazio é consequência de um aviso anterior; não duplicar.
        if let (Err(err), false) = (res, is_dir) {
            warn(w, tf("warn.blocked", &[&p.display(), &err]));
        }
    }
    freed
}

/// Se `path` é uma pasta `<nome>.avd`, devolve o arquivo `<nome>.ini` irmão, se existir.
fn avd_ini_sibling(path: &Path) -> Option<std::path::PathBuf> {
    let ext = path.extension()?.to_str()?;
    if !ext.eq_ignore_ascii_case("avd") {
        return None;
    }
    let ini = path.with_extension("ini");
    ini.is_file().then_some(ini)
}

/// Descarta alvos aninhados dentro de outro alvo também selecionado.
fn drop_nested(targets: &[Target]) -> Vec<&Target> {
    targets
        .iter()
        .filter(|t| {
            !targets
                .iter()
                .any(|o| o.path != t.path && t.path.starts_with(&o.path))
        })
        .collect()
}

/// Remove todos os alvos e devolve o relatório. `on_progress(concluídos, total)`.
pub fn clean<F>(
    targets: &[Target],
    w: &Warnings,
    opts: &CleanOptions,
    on_progress: F,
) -> CleanReport
where
    F: Fn(usize, usize) + Sync,
{
    let list: Vec<&Target> = drop_nested(targets)
        .into_iter()
        .filter(|t| {
            if t.kind == TargetKind::Dir && safety::is_protected(&t.path) {
                warn(w, tf("warn.protected", &[&t.path.display()]));
                false
            } else {
                true
            }
        })
        .collect();
    let total = list.len();
    let done = AtomicUsize::new(0);
    let removed = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);

    let work = || -> u64 {
        list.par_iter()
            .map(|t| {
                let mut freed = 0;
                if opts.cancel.is_cancelled() {
                    skipped.fetch_add(1, Ordering::Relaxed);
                } else if t.kind != TargetKind::Dir {
                    // Docker: o próprio Docker remove (não existe lixeira para isso).
                    match crate::docker::prune(t.kind) {
                        Ok(()) => {
                            freed = t.size;
                            removed.fetch_add(1, Ordering::Relaxed);
                            tracing::info!(target = %t.name, size = t.size, "docker pruned");
                        }
                        Err(e) => {
                            skipped.fetch_add(1, Ordering::Relaxed);
                            warn(w, tf("warn.docker", &[&t.name, &e]));
                        }
                    }
                } else if !t.path.exists() {
                    // Já removido (ex.: era filho de outro item).
                    skipped.fetch_add(1, Ordering::Relaxed);
                } else {
                    let avd_ini = avd_ini_sibling(&t.path);
                    match opts.mode {
                        DeleteMode::Trash => match trash::delete(&t.path) {
                            Ok(()) => {
                                if let Some(ini) = avd_ini {
                                    let _ = trash::delete(ini);
                                }
                                freed = t.size;
                                removed.fetch_add(1, Ordering::Relaxed);
                                tracing::info!(path = %t.path.display(), size = t.size, "moved to trash");
                            }
                            Err(e) => {
                                skipped.fetch_add(1, Ordering::Relaxed);
                                warn(w, tf("warn.trash", &[&t.path.display(), &e]));
                            }
                        },
                        DeleteMode::Permanent => {
                            freed = remove_tree(&t.path, w, &opts.cancel);
                            if let Some(ini) = avd_ini {
                                let _ = remove_file_force(&ini);
                            }
                            removed.fetch_add(1, Ordering::Relaxed);
                            tracing::info!(path = %t.path.display(), freed, "deleted permanently");
                        }
                    }
                }
                on_progress(done.fetch_add(1, Ordering::Relaxed) + 1, total);
                freed
            })
            .sum()
    };
    let freed = match rayon::ThreadPoolBuilder::new()
        .num_threads(opts.threads.max(1))
        .build()
    {
        Ok(pool) => pool.install(work),
        Err(_) => work(),
    };
    CleanReport {
        freed,
        removed: removed.into_inner(),
        skipped: skipped.into_inner(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use std::path::PathBuf;

    fn target(p: PathBuf, size: u64) -> Target {
        let mut t = Target::new(Category::Tool, "t", p, false);
        t.size = size;
        t
    }

    #[test]
    fn permanent_removes_tree_including_readonly_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("victim");
        fs::create_dir_all(root.join("a")).unwrap();
        let ro = root.join("a").join("ro.txt");
        fs::write(&ro, vec![0u8; 100]).unwrap();
        let mut perm = fs::metadata(&ro).unwrap().permissions();
        perm.set_readonly(true);
        fs::set_permissions(&ro, perm).unwrap();
        fs::write(root.join("b.txt"), vec![0u8; 50]).unwrap();

        let w = Warnings::default();
        let opts = CleanOptions {
            mode: DeleteMode::Permanent,
            ..Default::default()
        };
        let rep = clean(&[target(root.clone(), 150)], &w, &opts, |_, _| {});
        assert!(!root.exists());
        assert_eq!(rep.freed, 150);
        assert_eq!(rep.removed, 1);
        assert!(w.lock().unwrap().is_empty());
    }

    #[test]
    fn nested_targets_are_deduplicated() {
        let dir = tempfile::tempdir().unwrap();
        let outer = dir.path().join("outer");
        let inner = outer.join("inner");
        fs::create_dir_all(&inner).unwrap();
        fs::write(inner.join("f"), b"hello").unwrap();
        let list = [target(outer.clone(), 5), target(inner, 5)];
        assert_eq!(drop_nested(&list).len(), 1);
        let w = Warnings::default();
        let opts = CleanOptions {
            mode: DeleteMode::Permanent,
            ..Default::default()
        };
        let rep = clean(&list, &w, &opts, |_, _| {});
        assert!(!outer.exists());
        assert_eq!(rep.removed, 1);
    }

    #[test]
    fn protected_paths_are_refused() {
        let w = Warnings::default();
        let home = dirs::home_dir().unwrap();
        let opts = CleanOptions {
            mode: DeleteMode::Permanent,
            ..Default::default()
        };
        let rep = clean(&[target(home.clone(), 1)], &w, &opts, |_, _| {});
        assert_eq!(rep.removed, 0);
        assert!(home.exists());
        assert_eq!(w.lock().unwrap().len(), 1);
    }

    #[test]
    fn cancelled_clean_removes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("keep");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("f"), b"x").unwrap();
        let opts = CleanOptions {
            mode: DeleteMode::Permanent,
            ..Default::default()
        };
        opts.cancel.cancel();
        let rep = clean(
            &[target(root.clone(), 1)],
            &Warnings::default(),
            &opts,
            |_, _| {},
        );
        assert!(root.exists());
        assert_eq!(rep.skipped, 1);
    }
}
