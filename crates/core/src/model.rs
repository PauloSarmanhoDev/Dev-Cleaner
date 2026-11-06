use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Language,
    Ide,
    LocalProject,
}

impl Category {
    pub fn label(self) -> &'static str {
        match self {
            Category::Language => "Linguagem",
            Category::Ide => "IDE/Editor",
            Category::LocalProject => "Projeto Local",
        }
    }
}

/// Um diretório candidato à limpeza.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub category: Category,
    pub name: String,
    pub path: PathBuf,
    /// Exige confirmação explícita extra (ex.: extensões de IDEs).
    pub critical: bool,
    pub size: u64,
    pub files: u64,
}

impl Target {
    pub fn new(category: Category, name: impl Into<String>, path: PathBuf, critical: bool) -> Self {
        Self { category, name: name.into(), path, critical, size: 0, files: 0 }
    }
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



