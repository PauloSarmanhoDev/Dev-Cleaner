//! Funções compartilhadas por CLI e GUI: montagem da lista de candidatos e filtros.

use crate::config::Config;
use crate::i18n::tf;
use crate::model::{Target, now_secs};
use crate::{Warnings, discovery, safety};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Selection {
    pub lang: bool,
    pub ide: bool,
    /// Se `None`, projetos locais não são procurados.
    pub root: Option<PathBuf>,
    pub depth: usize,
}

/// Reúne os diretórios candidatos, aplicando alvos extras e exclusões da configuração.
pub fn candidates(sel: &Selection, cfg: &Config, w: &Warnings) -> Vec<Target> {
    let mut out = Vec::new();
    if sel.lang {
        out.extend(discovery::language_targets());
        out.extend(crate::docker::targets());
    }
    if sel.ide {
        out.extend(discovery::ide_targets());
    }
    if let Some(root) = &sel.root {
        if root.is_dir() {
            for t in discovery::local_targets(root, sel.depth.clamp(1, 20)) {
                if safety::git_tracked(&t.path) {
                    w.lock().unwrap().push(tf("warn.git", &[&t.path.display()]));
                } else {
                    out.push(t);
                }
            }
        } else {
            w.lock()
                .unwrap()
                .push(tf("warn.bad_root", &[&root.display()]));
        }
    }
    out.extend(cfg.extra_targets());

    let mut seen = std::collections::HashSet::new();
    out.retain(|t| !cfg.is_excluded(&t.path) && seen.insert(t.path.clone()));
    out
}

/// Mantém apenas alvos sem uso há pelo menos `min_age_secs`. Sem data conhecida = mantém.
pub fn filter_older_than(targets: Vec<Target>, min_age_secs: u64) -> Vec<Target> {
    let limit = now_secs().saturating_sub(min_age_secs);
    targets
        .into_iter()
        .filter(|t| t.last_used.is_none_or(|u| u <= limit))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;

    #[test]
    fn age_filter_keeps_old_and_unknown() {
        let now = now_secs();
        let mk = |used: Option<u64>| {
            let mut t = Target::new(Category::Tool, "x", PathBuf::from("x"), false);
            t.last_used = used;
            t
        };
        let out = filter_older_than(
            vec![mk(Some(now)), mk(Some(now - 40 * 86_400)), mk(None)],
            30 * 86_400,
        );
        assert_eq!(out.len(), 2);
    }
}
