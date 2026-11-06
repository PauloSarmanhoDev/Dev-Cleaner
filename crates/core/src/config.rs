//! Arquivo de configuração opcional (`dev-cleaner.toml`).
//!
//! ```toml
//! language = "pt"          # ou "en"
//! older_than = "30d"       # filtro de idade padrão
//! permanent = false        # true = não usar a lixeira
//! exclude = ["~/.cargo/registry", "D:/importante"]
//!
//! [[extra]]
//! name = "Meu cache"
//! path = "~/.meu-cache"
//! category = "tool"        # language | tool | ide | local
//! critical = false
//! ```

use crate::i18n::{Lang, tf};
use crate::model::{Category, Target};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub language: Option<String>,
    pub older_than: Option<String>,
    pub permanent: bool,
    pub exclude: Vec<String>,
    pub extra: Vec<ExtraTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraTarget {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub critical: bool,
    pub category: Option<String>,
}

pub fn default_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("dev-cleaner").join("config.toml"))
}

fn expand(p: &str) -> PathBuf {
    if let (Some(rest), Some(h)) = (
        p.strip_prefix("~/").or_else(|| p.strip_prefix("~\\")),
        dirs::home_dir(),
    ) {
        return h.join(rest);
    }
    PathBuf::from(p)
}

/// Grava a configuração (cria a pasta se preciso). O arquivo é reescrito por inteiro:
/// comentários feitos à mão são perdidos.
pub fn save(path: Option<&Path>, cfg: &Config) -> Result<PathBuf, String> {
    let p = path
        .map(Path::to_path_buf)
        .or_else(default_path)
        .ok_or_else(|| crate::i18n::t("err.cfg_nopath").to_string())?;
    let fail = |e: &dyn std::fmt::Display| tf("err.cfg_write", &[&p.display(), e]);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| fail(&e))?;
    }
    let text = toml::to_string_pretty(cfg).map_err(|e| fail(&e))?;
    std::fs::write(&p, text).map_err(|e| fail(&e))?;
    Ok(p)
}

/// Carrega a configuração. Arquivo ausente = padrão; erro de parse vira aviso textual.
pub fn load(path: Option<&Path>) -> (Config, Option<String>) {
    let Some(p) = path.map(Path::to_path_buf).or_else(default_path) else {
        return (Config::default(), None);
    };
    let Ok(text) = std::fs::read_to_string(&p) else {
        return (Config::default(), None);
    };
    match toml::from_str::<Config>(&text) {
        Ok(c) => (c, None),
        Err(e) => (
            Config::default(),
            Some(tf("warn.config", &[&p.display(), &e])),
        ),
    }
}

impl Config {
    /// Valida e normaliza (campos vazios viram None). Usada antes de salvar pela GUI.
    pub fn validate(&mut self) -> Result<(), String> {
        self.language = self
            .language
            .take()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty());
        if let Some(l) = &self.language
            && Lang::parse(l).is_none()
        {
            return Err(tf("err.cfg_lang", &[l]));
        }
        self.older_than = self
            .older_than
            .take()
            .map(|a| a.trim().to_string())
            .filter(|a| !a.is_empty());
        if let Some(a) = &self.older_than
            && crate::model::parse_age(a).is_none()
        {
            return Err(tf("err.cfg_age", &[a]));
        }
        self.exclude = self
            .exclude
            .iter()
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
            .collect();
        for e in &mut self.extra {
            e.name = e.name.trim().to_string();
            e.path = e.path.trim().to_string();
            if e.name.is_empty() || e.path.is_empty() {
                return Err(tf("err.cfg_extra", &[&e.name]));
            }
            e.category = e
                .category
                .take()
                .map(|c| c.trim().to_lowercase())
                .filter(|c| !c.is_empty());
            if let Some(c) = &e.category
                && !matches!(c.as_str(), "language" | "tool" | "ide" | "local")
            {
                return Err(tf("err.cfg_cat", &[&e.name, c]));
            }
        }
        Ok(())
    }

    pub fn extra_targets(&self) -> Vec<Target> {
        self.extra
            .iter()
            .filter_map(|e| {
                let path = expand(&e.path);
                let cat = match e.category.as_deref().map(str::to_lowercase).as_deref() {
                    Some("language") => Category::Language,
                    Some("ide") => Category::Ide,
                    Some("local") => Category::LocalProject,
                    _ => Category::Tool,
                };
                path.is_dir()
                    .then(|| Target::new(cat, e.name.clone(), path, e.critical))
            })
            .collect()
    }

    /// `true` se `path` é (ou está dentro de) um caminho excluído.
    pub fn is_excluded(&self, path: &Path) -> bool {
        self.exclude.iter().any(|e| path.starts_with(expand(e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_excludes() {
        let dir = tempfile::tempdir().unwrap();
        let extra = dir.path().join("cache");
        std::fs::create_dir(&extra).unwrap();
        let toml = format!(
            "language='en'\nexclude=['{0}/keep']\n[[extra]]\nname='X'\npath='{0}/cache'\ncritical=true\n[[extra]]\nname='Missing'\npath='{0}/nope'\n",
            dir.path().display().to_string().replace('\\', "/")
        );
        let file = dir.path().join("c.toml");
        std::fs::write(&file, toml).unwrap();
        let (cfg, err) = load(Some(&file));
        assert!(err.is_none());
        assert_eq!(cfg.language.as_deref(), Some("en"));
        let ex = cfg.extra_targets();
        assert_eq!(ex.len(), 1);
        assert!(ex[0].critical);
        assert!(cfg.is_excluded(&dir.path().join("keep").join("sub")));
        assert!(!cfg.is_excluded(&dir.path().join("other")));
    }

    #[test]
    fn save_roundtrip_and_validation() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("sub").join("c.toml");
        let mut cfg = Config {
            language: Some(" pt ".into()),
            older_than: Some("  ".into()),
            permanent: true,
            exclude: vec!["~/x".into(), "  ".into()],
            extra: vec![ExtraTarget {
                name: "N".into(),
                path: "~/p".into(),
                critical: true,
                category: Some("IDE".into()),
            }],
        };
        cfg.validate().unwrap();
        assert_eq!(cfg.older_than, None);
        assert_eq!(cfg.exclude, vec!["~/x"]);
        save(Some(&file), &cfg).unwrap();
        let (back, err) = load(Some(&file));
        assert!(err.is_none());
        assert_eq!(back.language.as_deref(), Some("pt"));
        assert!(back.permanent);
        assert_eq!(back.extra[0].category.as_deref(), Some("ide"));
        let mut bad = Config {
            language: Some("fr".into()),
            ..Default::default()
        };
        assert!(bad.validate().is_err());
        let mut bad = Config {
            older_than: Some("xx".into()),
            ..Default::default()
        };
        assert!(bad.validate().is_err());
    }

    #[test]
    fn bad_toml_becomes_warning() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("c.toml");
        std::fs::write(&file, "language = [").unwrap();
        let (cfg, err) = load(Some(&file));
        assert!(err.is_some());
        assert!(cfg.extra.is_empty());
    }
}
