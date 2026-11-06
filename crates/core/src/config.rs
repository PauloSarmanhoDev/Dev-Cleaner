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

use crate::i18n::tf;
use crate::model::{Category, Target};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub language: Option<String>,
    pub older_than: Option<String>,
    pub permanent: bool,
    pub exclude: Vec<String>,
    pub extra: Vec<ExtraTarget>,
}

#[derive(Debug, Deserialize)]
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
    fn bad_toml_becomes_warning() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("c.toml");
        std::fs::write(&file, "language = [").unwrap();
        let (cfg, err) = load(Some(&file));
        assert!(err.is_some());
        assert!(cfg.extra.is_empty());
    }
}
