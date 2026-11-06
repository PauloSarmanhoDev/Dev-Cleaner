use crate::i18n::t;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Language,
    Tool,
    Ide,
    LocalProject,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Language => t("cat.language"),
            Category::Tool => t("cat.tool"),
            Category::Ide => t("cat.ide"),
            Category::LocalProject => t("cat.local"),
        }
    }
}

/// Como o alvo é removido.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TargetKind {
    /// Diretório no disco (lixeira ou remoção permanente).
    #[default]
    Dir,
    /// Imagens Docker sem uso (docker image prune -a). Não passa pela lixeira.
    DockerImages,
    /// Volumes Docker sem uso (docker volume prune -a). Não passa pela lixeira.
    DockerVolumes,
}

/// Um diretório candidato à limpeza.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub category: Category,
    pub name: String,
    pub path: PathBuf,
    /// Exige confirmação explícita extra (ex.: extensões de IDEs).
    pub critical: bool,
    /// Tipo de remoção. Alvos que não são Dir usam o caminho apenas como identificador.
    #[serde(default)]
    pub kind: TargetKind,
    pub size: u64,
    pub files: u64,
    /// Modificação mais recente (segundos Unix) de qualquer arquivo dentro.
    #[serde(default)]
    pub last_used: Option<u64>,
    /// Nomes de processo (normalizados) da IDE dona do diretório.
    #[serde(default)]
    pub apps: Vec<String>,
    /// Preenchido quando algum dos `apps` está em execução.
    #[serde(default)]
    pub running: Option<String>,
}

impl Target {
    pub fn new(category: Category, name: impl Into<String>, path: PathBuf, critical: bool) -> Self {
        Self {
            category,
            name: name.into(),
            path,
            critical,
            kind: TargetKind::Dir,
            size: 0,
            files: 0,
            last_used: None,
            apps: Vec::new(),
            running: None,
        }
    }

    pub fn with_kind(mut self, kind: TargetKind) -> Self {
        self.kind = kind;
        self
    }

    /// Nome traduzido para o idioma atual (o campo `name` guarda o texto canônico em inglês).
    pub fn display_name(&self) -> String {
        crate::i18n::localize_name(&self.name)
    }

    pub fn with_apps(mut self, apps: &[&str]) -> Self {
        self.apps = apps.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Dias desde o último uso, se conhecido.
    pub fn age_days(&self) -> Option<u64> {
        self.last_used
            .map(|t| now_secs().saturating_sub(t) / 86_400)
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

/// Interpreta "30d", "2w", "12h", "90m", "45s" (número puro = dias) em segundos.
pub fn parse_age(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    let (num, mult) = match s.chars().last()? {
        'd' => (&s[..s.len() - 1], 86_400),
        'w' => (&s[..s.len() - 1], 604_800),
        'h' => (&s[..s.len() - 1], 3_600),
        'm' => (&s[..s.len() - 1], 60),
        's' => (&s[..s.len() - 1], 1),
        c if c.is_ascii_digit() => (s.as_str(), 86_400),
        _ => return None,
    };
    num.trim().parse::<u64>().ok().map(|n| n * mult)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_sizes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(2048), "2 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_size(3 * 1024 * 1024 * 1024), "3.00 GB");
    }

    #[test]
    fn parses_ages() {
        assert_eq!(parse_age("30d"), Some(30 * 86_400));
        assert_eq!(parse_age("2w"), Some(14 * 86_400));
        assert_eq!(parse_age("12h"), Some(43_200));
        assert_eq!(parse_age("7"), Some(7 * 86_400));
        assert_eq!(parse_age("abc"), None);
        assert_eq!(parse_age(""), None);
    }
}
