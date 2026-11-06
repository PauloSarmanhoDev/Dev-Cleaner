//! Regras de segurança: caminhos protegidos, rastreamento git e confirmação de itens críticos.

use crate::i18n::{t, tf};
use crate::model::Target;
use std::path::{Component, Path};
use std::process::Command;

/// Palavras aceitas para confirmar a remoção de itens críticos.
pub const CRITICAL_PHRASES: &[&str] = &["APAGAR", "DELETE"];

/// Caminhos que jamais devem ser removidos: raiz, home, pastas-base do usuário
/// (e qualquer ancestral delas).
pub fn is_protected(path: &Path) -> bool {
    let normal = path
        .components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count();
    if normal < 2 {
        return true;
    }
    let guarded = [
        dirs::home_dir(),
        dirs::config_dir(),
        dirs::cache_dir(),
        dirs::data_dir(),
        dirs::data_local_dir(),
        dirs::document_dir(),
        dirs::desktop_dir(),
        dirs::download_dir(),
    ];
    guarded.into_iter().flatten().any(|g| g.starts_with(path))
}

/// `true` se o git rastreia algum arquivo dentro de `path` (ex.: `target` commitado de propósito).
pub fn git_tracked(path: &Path) -> bool {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return false;
    };
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(parent).args(["ls-files", "--"]).arg(name);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

/// Valida a confirmação digitada quando há itens críticos entre os escolhidos.
pub fn check_critical(chosen: &[&Target], phrase: Option<&str>) -> Result<(), String> {
    if chosen.iter().any(|t| t.critical)
        && !phrase
            .map(str::trim)
            .is_some_and(|p| CRITICAL_PHRASES.contains(&p))
    {
        return Err(tf("err.critical", &[&t("critical.word")]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Category;
    use std::path::PathBuf;

    fn tgt(critical: bool) -> Target {
        Target::new(Category::Ide, "x", PathBuf::from("x"), critical)
    }

    #[test]
    fn protects_roots_and_home() {
        assert!(is_protected(Path::new("/")));
        assert!(is_protected(Path::new("C:\\")));
        if let Some(h) = dirs::home_dir() {
            assert!(is_protected(&h));
            assert!(is_protected(h.parent().unwrap()));
            assert!(!is_protected(&h.join(".cargo").join("registry")));
        }
    }

    #[test]
    fn critical_requires_phrase() {
        let c = tgt(true);
        let n = tgt(false);
        assert!(check_critical(&[&c], None).is_err());
        assert!(check_critical(&[&c], Some("nope")).is_err());
        assert!(check_critical(&[&c], Some(" APAGAR ")).is_ok());
        assert!(check_critical(&[&c], Some("DELETE")).is_ok());
        assert!(check_critical(&[&n], None).is_ok());
    }
}
