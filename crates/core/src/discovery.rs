//! Descoberta de diretórios candidatos (globais e locais ao projeto).

use crate::model::{Category, Target};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn push(out: &mut Vec<Target>, cat: Category, name: &str, path: Option<PathBuf>, critical: bool) {
    if let Some(p) = path {
        if p.is_dir() {
            out.push(Target::new(cat, name, p, critical));
        }
    }
}

/// Subdiretórios imediatos de `dir` cujo nome satisfaz `pred`.
fn subdirs(dir: &Path, pred: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir() && pred(&e.file_name().to_string_lossy()))
        .map(|e| e.path())
        .collect()
}

/// Caches de linguagens/gerenciadores de pacotes.
pub fn language_targets() -> Vec<Target> {
    let mut t = Vec::new();
    let home = dirs::home_dir();
    let cache = dirs::cache_dir(); // Linux ~/.cache | macOS ~/Library/Caches | Win %LOCALAPPDATA%
    let local = dirs::data_local_dir();
    let h = |p: &str| home.as_ref().map(|h| h.join(p));
    let c = |p: &str| cache.as_ref().map(|c| c.join(p));
    let l = |p: &str| local.as_ref().map(|l| l.join(p));
    use Category::Language as L;

    // Rust
    push(&mut t, L, "Cargo registry", h(".cargo/registry"), false);
    push(&mut t, L, "Cargo git", h(".cargo/git"), false);
    // Node
    push(&mut t, L, "npm cache", h(".npm/_cacache"), false);
    push(&mut t, L, "npm cache", l("npm-cache"), false);
    push(&mut t, L, "pnpm store", l("pnpm/store"), false);
    push(&mut t, L, "pnpm store", h(".local/share/pnpm/store"), false);
    push(&mut t, L, "pnpm store", h("Library/pnpm/store"), false);
    push(&mut t, L, "Yarn cache", c("yarn"), false);
    push(&mut t, L, "Yarn cache", c("Yarn/Cache"), false);
    push(&mut t, L, "Yarn cache", l("Yarn/Cache"), false);
    // Python
    push(&mut t, L, "pip cache", c("pip"), false);
    push(&mut t, L, "Poetry cache", c("pypoetry"), false);
    push(&mut t, L, "uv cache", c("uv"), false);
    // Java
    push(&mut t, L, "Maven repository", h(".m2/repository"), false);
    push(&mut t, L, "Gradle caches", h(".gradle/caches"), false);
    // Go
    push(&mut t, L, "Go module cache", h("go/pkg/mod"), false);
    push(&mut t, L, "Go build cache", c("go-build"), false);
    // .NET
    push(&mut t, L, "NuGet packages", h(".nuget/packages"), false);

    dedup(t)
}

/// Extensões, caches e logs de IDEs/editores.
pub fn ide_targets() -> Vec<Target> {
    let mut t = Vec::new();
    use Category::Ide as I;

    if let Some(home) = dirs::home_dir() {
        for (name, dir) in [
            ("VS Code extensões", ".vscode/extensions"),
            ("VS Code Insiders extensões", ".vscode-insiders/extensions"),
        ] {
            push(&mut t, I, name, Some(home.join(dir)), true); // crítico
        }
    }

    // config_dir: Linux ~/.config | macOS ~/Library/Application Support | Win %APPDATA%
    if let Some(cfg) = dirs::config_dir() {
        for product in ["Code", "Code - Insiders"] {
            for sub in ["CachedData", "logs", "Cache", "CachedExtensionVSIXs", "Code Cache"] {
                push(
                    &mut t,
                    I,
                    &format!("{product}/{sub}"),
                    Some(cfg.join(product).join(sub)),
                    false,
                );
            }
        }
    }

    if let Some(cache) = dirs::cache_dir() {
        // JetBrains: um alvo por produto/versão (IntelliJIdea2024.1, WebStorm..., etc.)
        let jb = cache.join("JetBrains");
        for p in subdirs(&jb, |_| true) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            t.push(Target::new(I, format!("JetBrains {n}"), p, false));
        }
        // Android Studio (Google/AndroidStudioX.Y)
        for p in subdirs(&cache.join("Google"), |n| n.starts_with("AndroidStudio")) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            t.push(Target::new(I, format!("Android Studio {n}"), p, false));
        }
    }
    dedup(t)
}

fn dedup(v: Vec<Target>) -> Vec<Target> {
    let mut seen = HashSet::new();
    v.into_iter().filter(|t| seen.insert(t.path.clone())).collect()
}

/// Nomes de artefatos que nunca devem ser percorridos durante a busca por projetos.
const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", ".venv", "venv", "__pycache__", "bin", "obj", "build",
    ".gradle", ".cache", "AppData", "Library", "Applications",
];

/// Procura projetos sob `root` e devolve seus artefatos de build/dependências.
pub fn local_targets(root: &Path, max_depth: usize) -> Vec<Target> {
    let mut out = Vec::new();
    let walker = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && e.depth() > 0
                && SKIP_DIRS.iter().any(|s| e.file_name() == *s))
        });
    for entry in walker.flatten().filter(|e| e.file_type().is_dir()) {
        let dir = entry.path();
        let has = |f: &str| dir.join(f).is_file();
        let mut art: Vec<(&str, &str)> = Vec::new();
        if has("Cargo.toml") {
            art.push(("Rust", "target"));
        }
        if has("package.json") {
            art.push(("Node", "node_modules"));
        }
        if has("pom.xml") {
            art.push(("Maven", "target"));
        }
        if has("build.gradle") || has("build.gradle.kts") {
            art.push(("Gradle", "build"));
            art.push(("Gradle", ".gradle"));
        }
        if has("pyproject.toml") || has("requirements.txt") {
            art.push(("Python", ".venv"));
            art.push(("Python", "venv"));
        }
        let dotnet = std::fs::read_dir(dir).into_iter().flatten().flatten().any(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            n.ends_with(".csproj") || n.ends_with(".fsproj") || n.ends_with(".sln")
        });
        if dotnet {
            art.push((".NET", "bin"));
            art.push((".NET", "obj"));
        }
        for (kind, folder) in art {
            let p = dir.join(folder);
            if p.is_dir() {
                let proj = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                out.push(Target::new(Category::LocalProject, format!("{kind}: {proj}/{folder}"), p, false));
            }
        }
    }
    dedup(out)
}

