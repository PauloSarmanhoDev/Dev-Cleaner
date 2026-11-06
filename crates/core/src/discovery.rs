//! Descoberta de diretórios candidatos (globais e locais ao projeto).

use crate::model::{Category, Target};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn push(out: &mut Vec<Target>, cat: Category, name: &str, path: Option<PathBuf>) {
    if let Some(p) = path.filter(|p| p.is_dir()) {
        out.push(Target::new(cat, name, p, false));
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

fn dedup(v: Vec<Target>) -> Vec<Target> {
    let mut seen = std::collections::HashSet::new();
    v.into_iter()
        .filter(|t| seen.insert(t.path.clone()))
        .collect()
}

/// Caches de linguagens/gerenciadores de pacotes e ferramentas de desenvolvimento.
pub fn language_targets() -> Vec<Target> {
    let mut t = Vec::new();
    let home = dirs::home_dir();
    let cache = dirs::cache_dir(); // Linux ~/.cache | macOS ~/Library/Caches | Win %LOCALAPPDATA%
    let local = dirs::data_local_dir();
    let h = |p: &str| home.as_ref().map(|h| h.join(p));
    let c = |p: &str| cache.as_ref().map(|c| c.join(p));
    let l = |p: &str| local.as_ref().map(|l| l.join(p));
    use Category::{Language as L, Tool as T};

    // Rust
    push(&mut t, L, "Cargo registry", h(".cargo/registry"));
    push(&mut t, L, "Cargo git", h(".cargo/git"));
    // Node / JS
    push(&mut t, L, "npm cache", h(".npm/_cacache"));
    push(&mut t, L, "npm cache", l("npm-cache"));
    push(&mut t, L, "pnpm store", l("pnpm/store"));
    push(&mut t, L, "pnpm store", h(".local/share/pnpm/store"));
    push(&mut t, L, "pnpm store", h("Library/pnpm/store"));
    push(&mut t, L, "Yarn cache", c("yarn"));
    push(&mut t, L, "Yarn cache", c("Yarn/Cache"));
    push(&mut t, L, "Yarn cache", l("Yarn/Cache"));
    push(&mut t, L, "Bun cache", h(".bun/install/cache"));
    push(&mut t, L, "Deno cache", c("deno"));
    // Python
    push(&mut t, L, "pip cache", c("pip"));
    push(&mut t, L, "Poetry cache", c("pypoetry"));
    push(&mut t, L, "uv cache", c("uv"));
    // Java
    push(&mut t, L, "Maven repository", h(".m2/repository"));
    push(&mut t, L, "Gradle caches", h(".gradle/caches"));
    push(
        &mut t,
        L,
        "Gradle wrapper dists",
        h(".gradle/wrapper/dists"),
    );
    // Go
    push(&mut t, L, "Go module cache", h("go/pkg/mod"));
    push(&mut t, L, "Go build cache", c("go-build"));
    // .NET
    push(&mut t, L, "NuGet packages", h(".nuget/packages"));
    // Dart/Flutter, PHP, Ruby
    push(&mut t, L, "Dart/Flutter pub cache", h(".pub-cache"));
    push(&mut t, L, "Dart/Flutter pub cache", l("Pub/Cache"));
    push(&mut t, L, "Composer cache", c("composer"));
    push(&mut t, L, "Composer cache", c("Composer"));
    push(&mut t, L, "RubyGems", h(".gem"));
    // Ferramentas
    push(&mut t, T, "Homebrew cache", c("Homebrew"));
    push(
        &mut t,
        T,
        "Xcode DerivedData",
        h("Library/Developer/Xcode/DerivedData"),
    );
    push(
        &mut t,
        T,
        "Xcode iOS DeviceSupport",
        h("Library/Developer/Xcode/iOS DeviceSupport"),
    );
    push(&mut t, T, "Android SDK cache", h(".android/cache"));
    push(&mut t, T, "Docker buildx cache", h(".docker/buildx"));

    dedup(t)
}

/// Raízes de extensões (relativas ao home) de editores baseados em VS Code.
const EXTENSION_ROOTS: &[(&str, &str, &str)] = &[
    ("VS Code", ".vscode/extensions", "code"),
    (
        "VS Code Insiders",
        ".vscode-insiders/extensions",
        "code - insiders",
    ),
    ("Cursor", ".cursor/extensions", "cursor"),
    ("VSCodium", ".vscode-oss/extensions", "vscodium"),
];

/// Diretórios de dados (em `config_dir`) com caches desses editores.
const EDITOR_DATA_DIRS: &[(&str, &str)] = &[
    ("Code", "code"),
    ("Code - Insiders", "code - insiders"),
    ("Cursor", "cursor"),
    ("VSCodium", "vscodium"),
];

/// Nome de processo (normalizado) a partir do nome da pasta de produto da JetBrains.
fn jetbrains_app(dir_name: &str) -> Option<&'static str> {
    let n = dir_name.to_lowercase();
    [
        ("intellij", "idea"),
        ("idea", "idea"),
        ("webstorm", "webstorm"),
        ("pycharm", "pycharm"),
        ("androidstudio", "studio"),
        ("clion", "clion"),
        ("goland", "goland"),
        ("rider", "rider"),
        ("phpstorm", "phpstorm"),
        ("rubymine", "rubymine"),
        ("datagrip", "datagrip"),
        ("rustrover", "rustrover"),
        ("dataspell", "dataspell"),
        ("aqua", "aqua"),
    ]
    .iter()
    .find(|(prefix, _)| n.starts_with(prefix))
    .map(|(_, app)| *app)
}

/// Extensões, caches e logs de IDEs/editores.
pub fn ide_targets() -> Vec<Target> {
    let mut t = Vec::new();
    use Category::Ide as I;

    if let Some(home) = dirs::home_dir() {
        for (product, rel, app) in EXTENSION_ROOTS {
            let dir = home.join(rel);
            if !dir.is_dir() {
                continue;
            }
            // Versões antigas de extensões: seguras de remover (a mais recente fica).
            for old in old_extension_versions(&dir) {
                let n = old.file_name().unwrap().to_string_lossy().into_owned();
                t.push(
                    Target::new(I, format!("{product} old extension: {n}"), old, false)
                        .with_apps(&[app]),
                );
            }
            // Pasta inteira: crítica (todas as extensões serão perdidas).
            t.push(Target::new(I, format!("{product} extensions"), dir, true).with_apps(&[app]));
        }
    }

    // config_dir: Linux ~/.config | macOS ~/Library/Application Support | Win %APPDATA%
    if let Some(cfg) = dirs::config_dir() {
        for (product, app) in EDITOR_DATA_DIRS {
            for sub in [
                "CachedData",
                "logs",
                "Cache",
                "CachedExtensionVSIXs",
                "Code Cache",
            ] {
                let p = cfg.join(product).join(sub);
                if p.is_dir() {
                    t.push(Target::new(I, format!("{product}/{sub}"), p, false).with_apps(&[app]));
                }
            }
        }
    }

    if let Some(cache) = dirs::cache_dir() {
        // JetBrains: um alvo por produto/versão (IntelliJIdea2024.1, WebStorm..., etc.)
        for p in subdirs(&cache.join("JetBrains"), |_| true) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            let mut tg = Target::new(I, format!("JetBrains {n}"), p, false);
            if let Some(app) = jetbrains_app(&n) {
                tg = tg.with_apps(&[app]);
            }
            t.push(tg);
        }
        // Android Studio (Google/AndroidStudioX.Y)
        for p in subdirs(&cache.join("Google"), |n| n.starts_with("AndroidStudio")) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            t.push(Target::new(I, format!("Android Studio {n}"), p, false).with_apps(&["studio"]));
        }
        // Sublime Text e Zed
        for (name, rel, app) in [
            ("Sublime Text cache", "Sublime Text/Cache", "sublime_text"),
            ("Sublime Text cache", "sublime-text/Cache", "sublime_text"),
            ("Zed cache", "Zed", "zed"),
            ("Zed cache", "zed", "zed"),
        ] {
            let p = cache.join(rel);
            if p.is_dir() {
                t.push(Target::new(I, name, p, false).with_apps(&[app]));
            }
        }
    }
    if let Some(local) = dirs::data_local_dir() {
        let p = local.join("Zed");
        if p.is_dir() {
            t.push(Target::new(I, "Zed data", p, false).with_apps(&["zed"]));
        }
    }
    dedup(t)
}

/// "publisher.name-1.2.3-win32-x64" → ("publisher.name|win32-x64", [1,2,3]).
fn split_extension_dir(name: &str) -> Option<(String, Vec<u64>)> {
    let b = name.as_bytes();
    let i = (0..b.len().saturating_sub(1)).find(|&i| b[i] == b'-' && b[i + 1].is_ascii_digit())?;
    let id = &name[..i];
    let rest = &name[i + 1..];
    let (ver, platform) = rest.split_once('-').unwrap_or((rest, ""));
    let nums: Vec<u64> = ver
        .split('.')
        .map(|p| {
            p.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect();
    Some((format!("{id}|{platform}"), nums))
}

/// Versões antigas de extensões em `dir` (mantém a maior versão de cada extensão/plataforma).
pub fn old_extension_versions(dir: &Path) -> Vec<PathBuf> {
    let mut groups: HashMap<String, Vec<(Vec<u64>, PathBuf)>> = HashMap::new();
    for p in subdirs(dir, |n| !n.starts_with('.')) {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if let Some((key, ver)) = split_extension_dir(&name) {
            groups.entry(key).or_default().push((ver, p));
        }
    }
    let mut old = Vec::new();
    for (_, mut v) in groups {
        v.sort_by(|a, b| b.0.cmp(&a.0)); // maior primeiro
        old.extend(v.into_iter().skip(1).map(|(_, p)| p));
    }
    old.sort();
    old
}

/// Nomes de artefatos que nunca devem ser percorridos durante a busca por projetos.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    "bin",
    "obj",
    "build",
    ".gradle",
    ".cache",
    "AppData",
    "Library",
    "Applications",
    ".dart_tool",
    "vendor",
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
        if has("pubspec.yaml") {
            art.push(("Dart", ".dart_tool"));
            art.push(("Dart", "build"));
        }
        if has("composer.json") {
            art.push(("PHP", "vendor"));
        }
        if has("Package.swift") {
            art.push(("Swift", ".build"));
        }
        let dotnet = std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.ends_with(".csproj") || n.ends_with(".fsproj") || n.ends_with(".sln")
            });
        if dotnet {
            art.push((".NET", "bin"));
            art.push((".NET", "obj"));
        }
        for (kind, folder) in art {
            let p = dir.join(folder);
            if p.is_dir() && !crate::safety::is_protected(&p) {
                let proj = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                out.push(Target::new(
                    Category::LocalProject,
                    format!("{kind}: {proj}/{folder}"),
                    p,
                    false,
                ));
            }
        }
    }
    dedup(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn splits_extension_dirs() {
        assert_eq!(
            split_extension_dir("ms-python.python-2024.1.2"),
            Some(("ms-python.python|".into(), vec![2024, 1, 2]))
        );
        assert_eq!(
            split_extension_dir("rust-lang.rust-analyzer-0.3.10-win32-x64"),
            Some(("rust-lang.rust-analyzer|win32-x64".into(), vec![0, 3, 10]))
        );
        assert_eq!(split_extension_dir("noversion"), None);
    }

    #[test]
    fn keeps_only_latest_extension_version() {
        let d = tempfile::tempdir().unwrap();
        for n in [
            "a.b-1.9.0",
            "a.b-1.10.0",
            "a.b-1.2.0",
            "c.d-2.0.0-win32-x64",
            "c.d-1.0.0-win32-x64",
            "solo.ext-1.0.0",
            ".obsolete",
        ] {
            fs::create_dir(d.path().join(n)).unwrap();
        }
        let old: Vec<String> = old_extension_versions(d.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        // 1.10.0 > 1.9.0 (comparação numérica, não textual)
        assert_eq!(old, vec!["a.b-1.2.0", "a.b-1.9.0", "c.d-1.0.0-win32-x64"]);
    }

    #[test]
    fn finds_local_artifacts_by_marker() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("proj");
        fs::create_dir_all(p.join("target")).unwrap();
        fs::create_dir_all(p.join("node_modules")).unwrap();
        fs::write(p.join("Cargo.toml"), "").unwrap();
        // sem package.json: node_modules não deve ser listado
        let found = local_targets(d.path(), 4);
        assert_eq!(found.len(), 1);
        assert!(found[0].path.ends_with("target"));
    }

    #[test]
    fn maps_jetbrains_products() {
        assert_eq!(jetbrains_app("IntelliJIdea2026.2"), Some("idea"));
        assert_eq!(jetbrains_app("AndroidStudio2024.1"), Some("studio"));
        assert_eq!(jetbrains_app("Daemon"), None);
    }
}
