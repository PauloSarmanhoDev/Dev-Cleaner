//! Descoberta de diretórios candidatos (globais e locais ao projeto).

use crate::model::{Category, Target};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn push(out: &mut Vec<Target>, cat: Category, cid: &str, name: &str, path: Option<PathBuf>) {
    push_opt(out, cat, cid, name, path, false);
}

fn push_crit(out: &mut Vec<Target>, cat: Category, cid: &str, name: &str, path: Option<PathBuf>) {
    push_opt(out, cat, cid, name, path, true);
}

fn push_opt(
    out: &mut Vec<Target>,
    cat: Category,
    cid: &str,
    name: &str,
    path: Option<PathBuf>,
    critical: bool,
) {
    if let Some(p) = path.filter(|p| p.is_dir()) {
        out.push(Target::new(cat, name, p, critical).with_catalog(cid));
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
    push(&mut t, L, "rust", "Cargo registry", h(".cargo/registry"));
    push(&mut t, L, "rust", "Cargo git", h(".cargo/git"));
    push_crit(
        &mut t,
        L,
        "rust",
        "Rustup toolchains",
        h(".rustup/toolchains"),
    );
    push(
        &mut t,
        L,
        "rust",
        "Rustup downloads",
        h(".rustup/downloads"),
    );
    // Node / JS
    push(&mut t, L, "npm", "npm cache", h(".npm/_cacache"));
    push(&mut t, L, "npm", "npm cache", l("npm-cache"));
    push(&mut t, L, "pnpm", "pnpm store", l("pnpm/store"));
    push(
        &mut t,
        L,
        "pnpm",
        "pnpm store",
        h(".local/share/pnpm/store"),
    );
    push(&mut t, L, "pnpm", "pnpm store", h("Library/pnpm/store"));
    push(&mut t, L, "yarn", "Yarn cache", c("yarn"));
    push(&mut t, L, "yarn", "Yarn cache", c("Yarn/Cache"));
    push(&mut t, L, "yarn", "Yarn cache", l("Yarn/Cache"));
    push(&mut t, L, "bun", "Bun cache", h(".bun/install/cache"));
    push(&mut t, L, "deno", "Deno cache", c("deno"));
    push(&mut t, L, "node-gyp", "node-gyp cache", c("node-gyp"));
    push(&mut t, L, "node-gyp", "node-gyp cache", h(".node-gyp"));
    push(&mut t, L, "node-gyp", "nvm cache", h(".nvm/.cache"));
    // Python
    push(&mut t, L, "pip", "pip cache", c("pip"));
    push(&mut t, L, "poetry", "Poetry cache", c("pypoetry"));
    push(&mut t, L, "uv", "uv cache", c("uv"));
    push(&mut t, L, "pip", "pdm cache", c("pdm"));
    push(&mut t, L, "pip", "pipenv cache", c("pipenv"));
    push(&mut t, L, "conda", "Conda packages", h(".conda/pkgs"));
    push(&mut t, L, "conda", "Conda packages", h("miniconda3/pkgs"));
    push(&mut t, L, "conda", "Conda packages", h("anaconda3/pkgs"));
    push(&mut t, L, "conda", "Conda packages", h("miniforge3/pkgs"));
    // Java / JVM (Scala, Kotlin, Clojure)
    push(&mut t, L, "maven", "Maven repository", h(".m2/repository"));
    push(&mut t, L, "gradle", "Gradle caches", h(".gradle/caches"));
    push(
        &mut t,
        L,
        "gradle",
        "Gradle wrapper dists",
        h(".gradle/wrapper/dists"),
    );
    push(
        &mut t,
        L,
        "gradle",
        "Gradle daemon logs",
        h(".gradle/daemon"),
    );
    push(&mut t, L, "scala", "sbt ivy2 cache", h(".ivy2/cache"));
    push(&mut t, L, "scala", "sbt boot", h(".sbt/boot"));
    push(&mut t, L, "scala", "Coursier cache", c("coursier"));
    push(&mut t, L, "scala", "Coursier cache", l("Coursier/Cache"));
    push(
        &mut t,
        L,
        "scala",
        "Coursier cache",
        h("Library/Caches/Coursier/v1"),
    );
    push(&mut t, L, "kotlin", "Kotlin Native", h(".konan"));
    // Go
    push(&mut t, L, "go", "Go module cache", h("go/pkg/mod"));
    push(&mut t, L, "go", "Go build cache", c("go-build"));
    // .NET
    push(&mut t, L, "nuget", "NuGet packages", h(".nuget/packages"));
    push(&mut t, L, "nuget", "NuGet http-cache", l("NuGet/v3-cache"));
    push(
        &mut t,
        L,
        "nuget",
        "NuGet http-cache",
        h(".local/share/NuGet/v3-cache"),
    );
    // Dart/Flutter, PHP, Ruby
    push(&mut t, L, "dart", "Dart/Flutter pub cache", h(".pub-cache"));
    push(&mut t, L, "dart", "Dart/Flutter pub cache", l("Pub/Cache"));
    push(&mut t, L, "composer", "Composer cache", c("composer"));
    push(&mut t, L, "composer", "Composer cache", c("Composer"));
    push(&mut t, L, "ruby", "RubyGems", h(".gem"));
    push(&mut t, L, "ruby", "Bundler cache", h(".bundle/cache"));
    // Elixir / Erlang, Haskell, Zig, Julia, Nim, C/C++
    push(&mut t, L, "elixir", "Hex packages", h(".hex/packages"));
    push(&mut t, L, "elixir", "Hex cache", c("hex"));
    push(&mut t, L, "elixir", "Mix archives", h(".mix/archives"));
    push(&mut t, L, "elixir", "Rebar3 cache", c("rebar3"));
    push(&mut t, L, "haskell", "Cabal packages", h(".cabal/packages"));
    push(&mut t, L, "haskell", "Cabal store", h(".cabal/store"));
    push(&mut t, L, "haskell", "Stack root", h(".stack"));
    push(&mut t, L, "haskell", "GHCup cache", h(".ghcup/cache"));
    push(&mut t, L, "zig", "Zig cache", c("zig"));
    push(
        &mut t,
        L,
        "julia",
        "Julia compiled cache",
        h(".julia/compiled"),
    );
    push(&mut t, L, "nim", "Nimble packages", h(".nimble/pkgs2"));
    push(&mut t, L, "cpp", "Conan data", h(".conan2/p"));
    push(&mut t, L, "cpp", "Conan (v1) data", h(".conan/data"));
    push(&mut t, L, "cpp", "vcpkg archives", l("vcpkg/archives"));
    push(&mut t, L, "cpp", "vcpkg archives", c("vcpkg/archives"));
    push(&mut t, L, "lua", "LuaRocks cache", c("luarocks"));
    push(&mut t, L, "perl", "CPANM build work", h(".cpanm/work"));
    // Ferramentas: pacotes / mobile / Apple
    push(&mut t, T, "homebrew", "Homebrew cache", c("Homebrew"));
    push(
        &mut t,
        T,
        "xcode",
        "Xcode DerivedData",
        h("Library/Developer/Xcode/DerivedData"),
    );
    push(
        &mut t,
        T,
        "xcode",
        "Xcode iOS DeviceSupport",
        h("Library/Developer/Xcode/iOS DeviceSupport"),
    );
    push(
        &mut t,
        T,
        "xcode",
        "Xcode watchOS DeviceSupport",
        h("Library/Developer/Xcode/watchOS DeviceSupport"),
    );
    push_crit(
        &mut t,
        T,
        "xcode",
        "Xcode Archives",
        h("Library/Developer/Xcode/Archives"),
    );
    push(
        &mut t,
        T,
        "xcode",
        "Xcode CoreSimulator caches",
        h("Library/Developer/CoreSimulator/Caches"),
    );
    push(
        &mut t,
        T,
        "cocoapods",
        "CocoaPods cache",
        h("Library/Caches/CocoaPods"),
    );
    push(
        &mut t,
        T,
        "cocoapods",
        "CocoaPods repos",
        h(".cocoapods/repos"),
    );
    push(
        &mut t,
        T,
        "cocoapods",
        "Swift Package Manager cache",
        h("Library/Caches/org.swift.swiftpm"),
    );
    push(
        &mut t,
        T,
        "cocoapods",
        "Swift Package Manager cache",
        c("org.swift.swiftpm"),
    );
    push(
        &mut t,
        T,
        "android-sdk",
        "Android SDK cache",
        h(".android/cache"),
    );
    push(
        &mut t,
        T,
        "android-sdk",
        "Android build-cache",
        h(".android/build-cache"),
    );
    push(&mut t, T, "expo", "Expo cache", h(".expo"));
    // Ferramentas: containers, navegadores de teste, compilação
    push(
        &mut t,
        T,
        "docker-buildx",
        "Docker buildx cache",
        h(".docker/buildx"),
    );
    push(
        &mut t,
        T,
        "browsers",
        "Playwright browsers",
        c("ms-playwright"),
    );
    push(
        &mut t,
        T,
        "browsers",
        "Playwright browsers",
        h("Library/Caches/ms-playwright"),
    );
    push(
        &mut t,
        T,
        "browsers",
        "Puppeteer browsers",
        h(".cache/puppeteer"),
    );
    push(&mut t, T, "browsers", "Cypress cache", c("Cypress"));
    push(&mut t, T, "electron", "Electron cache", c("electron"));
    push(
        &mut t,
        T,
        "electron",
        "Electron Builder cache",
        c("electron-builder"),
    );
    push(&mut t, T, "ccache", "ccache", c("ccache"));
    push(&mut t, T, "ccache", "sccache", c("sccache"));
    push(&mut t, T, "ccache", "sccache", l("Mozilla/sccache"));
    push(&mut t, T, "ccache", "Bazel cache", c("bazel"));
    push(&mut t, T, "monorepo", "pre-commit cache", c("pre-commit"));
    push(&mut t, T, "monorepo", "Turborepo cache", c("turbo"));
    push(&mut t, T, "monorepo", "Nx cache", c("nx"));
    // Ferramentas: Cloud / DevOps / VMs
    push(
        &mut t,
        T,
        "cloud",
        "Terraform plugin cache",
        h(".terraform.d/plugin-cache"),
    );
    push(
        &mut t,
        T,
        "cloud",
        "Kubernetes kubectl cache",
        h(".kube/cache"),
    );
    push(&mut t, T, "cloud", "Helm cache", c("helm"));
    push(&mut t, T, "cloud", "AWS CLI cache", h(".aws/cli/cache"));
    push_crit(&mut t, T, "vms", "Vagrant boxes", h(".vagrant.d/boxes"));
    push(&mut t, T, "vms", "Minikube cache", h(".minikube/cache"));
    // Ferramentas: IA / Machine Learning (modelos grandes -> críticos)
    push_crit(
        &mut t,
        T,
        "ai",
        "Hugging Face cache",
        h(".cache/huggingface"),
    );
    push_crit(&mut t, T, "ai", "Hugging Face cache", c("huggingface"));
    push_crit(&mut t, T, "ai", "Ollama models", h(".ollama/models"));
    push_crit(&mut t, T, "ai", "PyTorch cache", h(".cache/torch"));
    push_crit(&mut t, T, "ai", "PyTorch cache", c("torch"));
    push(&mut t, T, "ai", "Keras datasets/models", h(".keras"));
    push_crit(
        &mut t,
        T,
        "ai",
        "LM Studio models",
        h(".cache/lm-studio/models"),
    );
    push_crit(&mut t, T, "ai", "LM Studio models", h(".lmstudio/models"));

    t.extend(avd_targets());

    dedup(t)
}

/// Pasta dos emuladores Android (ANDROID_AVD_HOME, ANDROID_USER_HOME/avd ou ~/.android/avd).
fn avd_home() -> Option<PathBuf> {
    let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
    env("ANDROID_AVD_HOME")
        .or_else(|| env("ANDROID_USER_HOME").map(|p| p.join("avd")))
        .or_else(|| dirs::home_dir().map(|h| h.join(".android/avd")))
}

/// Emuladores Android (AVD): snapshots (seguro) e o AVD inteiro (crítico: perde o estado e os dados).
fn avd_targets() -> Vec<Target> {
    let mut t = Vec::new();
    let Some(home) = avd_home() else { return t };
    // O processo do emulador é emulator ou qemu-system-* (o sufixo 64 é removido por procs).
    const APPS: &[&str] = &["emulator", "qemu-system-x86_", "qemu-system-aarch"];
    for dir in subdirs(&home, |n| n.ends_with(".avd")) {
        let n = dir.file_name().unwrap().to_string_lossy().into_owned();
        let n = n.trim_end_matches(".avd");
        let snaps = dir.join("snapshots");
        if snaps.is_dir() {
            t.push(
                Target::new(
                    Category::Tool,
                    format!("Android AVD snapshots: {n}"),
                    snaps,
                    false,
                )
                .with_apps(APPS)
                .with_catalog("avd"),
            );
        }
        t.push(
            Target::new(Category::Tool, format!("Android AVD: {n}"), dir, true)
                .with_apps(APPS)
                .with_catalog("avd"),
        );
    }
    t
}

/// Raízes de extensões (relativas ao home) de editores baseados em VS Code:
/// `(produto, caminho, processo, id_catalogo)`.
const EXTENSION_ROOTS: &[(&str, &str, &str, &str)] = &[
    ("VS Code", ".vscode/extensions", "code", "vscode"),
    (
        "VS Code Insiders",
        ".vscode-insiders/extensions",
        "code - insiders",
        "vscode",
    ),
    ("Cursor", ".cursor/extensions", "cursor", "vscode-forks"),
    (
        "VSCodium",
        ".vscode-oss/extensions",
        "vscodium",
        "vscode-forks",
    ),
    (
        "Windsurf",
        ".windsurf/extensions",
        "windsurf",
        "vscode-forks",
    ),
    ("Trae", ".trae/extensions", "trae", "vscode-forks"),
];

/// Diretórios de dados (em `config_dir`) com caches desses editores: `(pasta, processo, id_catalogo)`.
const EDITOR_DATA_DIRS: &[(&str, &str, &str)] = &[
    ("Code", "code", "vscode"),
    ("Code - Insiders", "code - insiders", "vscode"),
    ("Cursor", "cursor", "vscode-forks"),
    ("VSCodium", "vscodium", "vscode-forks"),
    ("Windsurf", "windsurf", "vscode-forks"),
    ("Trae", "trae", "vscode-forks"),
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
        ("fleet", "fleet"),
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
        for (product, rel, app, cid) in EXTENSION_ROOTS {
            let dir = home.join(rel);
            if !dir.is_dir() {
                continue;
            }
            // Versões antigas de extensões: seguras de remover (a mais recente fica).
            for old in old_extension_versions(&dir) {
                let n = old.file_name().unwrap().to_string_lossy().into_owned();
                t.push(
                    Target::new(I, format!("{product} old extension: {n}"), old, false)
                        .with_apps(&[app])
                        .with_catalog(cid),
                );
            }
            // Pasta inteira: crítica (todas as extensões serão perdidas).
            t.push(
                Target::new(I, format!("{product} extensions"), dir, true)
                    .with_apps(&[app])
                    .with_catalog(cid),
            );
        }
    }

    // config_dir: Linux ~/.config | macOS ~/Library/Application Support | Win %APPDATA%
    if let Some(cfg) = dirs::config_dir() {
        for (product, app, cid) in EDITOR_DATA_DIRS {
            for sub in [
                "CachedData",
                "logs",
                "Cache",
                "CachedExtensionVSIXs",
                "Code Cache",
            ] {
                let p = cfg.join(product).join(sub);
                if p.is_dir() {
                    t.push(
                        Target::new(I, format!("{product}/{sub}"), p, false)
                            .with_apps(&[app])
                            .with_catalog(cid),
                    );
                }
            }
        }
    }

    if let Some(cache) = dirs::cache_dir() {
        // JetBrains: um alvo por produto/versão (IntelliJIdea2024.1, WebStorm..., etc.)
        for p in subdirs(&cache.join("JetBrains"), |_| true) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            let mut tg =
                Target::new(I, format!("JetBrains {n}"), p, false).with_catalog("jetbrains");
            if let Some(app) = jetbrains_app(&n) {
                tg = tg.with_apps(&[app]);
            }
            t.push(tg);
        }
        // Android Studio (Google/AndroidStudioX.Y)
        for p in subdirs(&cache.join("Google"), |n| n.starts_with("AndroidStudio")) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            t.push(
                Target::new(I, format!("Android Studio {n}"), p, false)
                    .with_apps(&["studio"])
                    .with_catalog("android-studio"),
            );
        }
        // Sublime Text, Zed, Neovim e Fleet
        for (name, rel, app, cid) in [
            (
                "Sublime Text cache",
                "Sublime Text/Cache",
                "sublime_text",
                "sublime",
            ),
            (
                "Sublime Text cache",
                "sublime-text/Cache",
                "sublime_text",
                "sublime",
            ),
            ("Zed cache", "Zed", "zed", "zed"),
            ("Zed cache", "zed", "zed", "zed"),
            ("Neovim cache", "nvim", "nvim", "neovim"),
            ("JetBrains Fleet cache", "Fleet", "fleet", "jetbrains"),
        ] {
            let p = cache.join(rel);
            if p.is_dir() {
                t.push(
                    Target::new(I, name, p, false)
                        .with_apps(&[app])
                        .with_catalog(cid),
                );
            }
        }
    }
    if let Some(local) = dirs::data_local_dir() {
        let p = local.join("Zed");
        if p.is_dir() {
            t.push(
                Target::new(I, "Zed data", p, false)
                    .with_apps(&["zed"])
                    .with_catalog("zed"),
            );
        }
        let nvim = local.join("nvim-data");
        if nvim.is_dir() {
            t.push(
                Target::new(I, "Neovim data", nvim, true)
                    .with_apps(&["nvim"])
                    .with_catalog("neovim"),
            );
        }
        // Visual Studio: ComponentModelCache e cache de pacotes por versão
        for p in subdirs(&local.join("Microsoft/VisualStudio"), |_| true) {
            let n = p.file_name().unwrap().to_string_lossy().into_owned();
            let cmc = p.join("ComponentModelCache");
            if cmc.is_dir() {
                t.push(
                    Target::new(
                        I,
                        format!("Visual Studio {n} ComponentModelCache"),
                        cmc,
                        false,
                    )
                    .with_apps(&["devenv"])
                    .with_catalog("visual-studio"),
                );
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let nvim_share = home.join(".local/share/nvim");
        if nvim_share.is_dir() {
            t.push(
                Target::new(I, "Neovim data", nvim_share, true)
                    .with_apps(&["nvim"])
                    .with_catalog("neovim"),
            );
        }
        let p2 = home.join(".p2/pool");
        if p2.is_dir() {
            t.push(
                Target::new(I, "Eclipse p2 bundle pool", p2, true)
                    .with_apps(&["eclipse"])
                    .with_catalog("eclipse"),
            );
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

/// Extensões marcadas para remoção no arquivo `.obsolete` do VS Code (`{"pub.nome-1.0.0": true}`).
fn read_obsolete(dir: &Path) -> HashSet<String> {
    let Ok(text) = std::fs::read_to_string(dir.join(".obsolete")) else {
        return HashSet::new();
    };
    let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(&text) else {
        return HashSet::new();
    };
    map.into_iter()
        .filter(|(_, v)| v.as_bool().unwrap_or(false))
        .map(|(k, _)| k.to_lowercase())
        .collect()
}

/// Versões antigas de extensões em `dir`: as marcadas em `.obsolete` e, entre as demais,
/// tudo menos a maior versão de cada extensão/plataforma.
pub fn old_extension_versions(dir: &Path) -> Vec<PathBuf> {
    let obsolete = read_obsolete(dir);
    let mut old = Vec::new();
    let mut groups: HashMap<String, Vec<(Vec<u64>, PathBuf)>> = HashMap::new();
    for p in subdirs(dir, |n| !n.starts_with('.')) {
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if obsolete.contains(&name.to_lowercase()) {
            old.push(p);
        } else if let Some((key, ver)) = split_extension_dir(&name) {
            groups.entry(key).or_default().push((ver, p));
        }
    }
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
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".nox",
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
    ".next",
    ".nuxt",
    ".angular",
    ".svelte-kit",
    ".turbo",
    ".parcel-cache",
    ".astro",
    ".output",
    "_build",
    "deps",
    ".stack-work",
    "dist-newstyle",
    ".zig-cache",
    "zig-out",
    ".terraform",
    "Pods",
    "Temp",
    "Intermediate",
    "DerivedDataCache",
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
        let has_any = |fs: &[&str]| fs.iter().any(|f| has(f));
        let mut art: Vec<(&str, &str, &str)> = Vec::new();
        if has("Cargo.toml") {
            art.push(("p-rust", "Rust", "target"));
        }
        if has("package.json") {
            art.push(("p-node", "Node", "node_modules"));
            art.push(("p-node", "Next.js", ".next"));
            art.push(("p-node", "Nuxt", ".nuxt"));
            art.push(("p-node", "Angular", ".angular"));
            art.push(("p-node", "SvelteKit", ".svelte-kit"));
            art.push(("p-node", "Astro", ".astro"));
            art.push(("p-node", "Nitro/Nuxt", ".output"));
            art.push(("p-node", "Turborepo", ".turbo"));
            art.push(("p-node", "Parcel", ".parcel-cache"));
        }
        if has("pom.xml") {
            art.push(("p-maven", "Maven", "target"));
        }
        if has("build.gradle") || has("build.gradle.kts") {
            art.push(("p-gradle", "Gradle", "build"));
            art.push(("p-gradle", "Gradle", ".gradle"));
        }
        if has("build.sbt") {
            art.push(("p-scala", "Scala (sbt)", "target"));
            art.push(("p-scala", "Scala (sbt)", "project/target"));
        }
        if has("project.clj") || has("deps.edn") {
            art.push(("p-clojure", "Clojure", "target"));
            art.push(("p-clojure", "Clojure", ".cpcache"));
        }
        if has_any(&["pyproject.toml", "requirements.txt", "setup.py", "Pipfile"]) {
            art.push(("p-python", "Python", ".venv"));
            art.push(("p-python", "Python", "venv"));
            art.push(("p-python", "Python", "__pycache__"));
            art.push(("p-python", "Python", ".pytest_cache"));
            art.push(("p-python", "Python", ".mypy_cache"));
            art.push(("p-python", "Python", ".ruff_cache"));
            art.push(("p-python", "Python", ".tox"));
            art.push(("p-python", "Python", ".nox"));
        }
        if has("pubspec.yaml") {
            art.push(("p-dart", "Dart", ".dart_tool"));
            art.push(("p-dart", "Dart", "build"));
        }
        if has("composer.json") {
            art.push(("p-php", "PHP", "vendor"));
        }
        if has("Gemfile") {
            art.push(("p-ruby", "Ruby", "vendor/bundle"));
        }
        if has("Package.swift") {
            art.push(("p-swift", "Swift", ".build"));
        }
        if has("Podfile") {
            art.push(("p-swift", "CocoaPods", "Pods"));
        }
        if has("mix.exs") {
            art.push(("p-elixir", "Elixir", "_build"));
            art.push(("p-elixir", "Elixir", "deps"));
        }
        if has("rebar.config") {
            art.push(("p-elixir", "Erlang", "_build"));
        }
        if has("stack.yaml") || has("cabal.project") {
            art.push(("p-haskell", "Haskell", ".stack-work"));
            art.push(("p-haskell", "Haskell", "dist-newstyle"));
        }
        if has("build.zig") {
            art.push(("p-zig", "Zig", "zig-out"));
            art.push(("p-zig", "Zig", ".zig-cache"));
        }
        if has("CMakeLists.txt") {
            art.push(("p-cmake", "CMake", "build"));
            for sub in subdirs(dir, |n| n.starts_with("cmake-build-")) {
                if let Some(n) = sub.file_name().map(|s| s.to_string_lossy().into_owned()) {
                    let proj = dir
                        .file_name()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    out.push(
                        Target::new(
                            Category::LocalProject,
                            format!("CMake: {proj}/{n}"),
                            sub,
                            false,
                        )
                        .with_catalog("p-cmake"),
                    );
                }
            }
        }
        // Unity: pasta com Assets/ e ProjectSettings/
        if dir.join("Assets").is_dir() && dir.join("ProjectSettings").is_dir() {
            art.push(("p-unity", "Unity", "Library"));
            art.push(("p-unity", "Unity", "Temp"));
            art.push(("p-unity", "Unity", "Obj"));
            art.push(("p-unity", "Unity", "Logs"));
        }
        let mut dotnet = false;
        let mut haskell_cabal = false;
        let mut terraform = false;
        let mut unreal = false;
        for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let n = e.file_name().to_string_lossy().to_lowercase();
            dotnet |= n.ends_with(".csproj") || n.ends_with(".fsproj") || n.ends_with(".sln");
            haskell_cabal |= n.ends_with(".cabal");
            terraform |= n.ends_with(".tf") || n == ".terraform.lock.hcl";
            unreal |= n.ends_with(".uproject");
        }
        if dotnet {
            art.push(("p-dotnet", ".NET", "bin"));
            art.push(("p-dotnet", ".NET", "obj"));
        }
        if haskell_cabal {
            art.push(("p-haskell", "Haskell", "dist-newstyle"));
        }
        if terraform {
            art.push(("p-terraform", "Terraform", ".terraform"));
        }
        if unreal {
            art.push(("p-unreal", "Unreal", "Intermediate"));
            art.push(("p-unreal", "Unreal", "Binaries"));
            art.push(("p-unreal", "Unreal", "DerivedDataCache"));
            art.push(("p-unreal", "Unreal", "Saved/Logs"));
        }
        for (cid, kind, folder) in art {
            let p = dir.join(folder);
            if p.is_dir() && !crate::safety::is_protected(&p) {
                let proj = dir
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                out.push(
                    Target::new(
                        Category::LocalProject,
                        format!("{kind}: {proj}/{folder}"),
                        p,
                        false,
                    )
                    .with_catalog(cid),
                );
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
    fn obsolete_file_marks_extensions_for_removal() {
        let d = tempfile::tempdir().unwrap();
        for n in ["a.b-1.10.0", "a.b-1.9.0", "c.d-1.0.0", "e.f-2.0.0"] {
            fs::create_dir(d.path().join(n)).unwrap();
        }
        // 1.10.0 está obsoleta: a versão mantida passa a ser 1.9.0. "c.d" com false e a
        // entrada de pasta inexistente são ignoradas.
        fs::write(
            d.path().join(".obsolete"),
            r#"{"a.b-1.10.0": true, "c.d-1.0.0": false, "gone-1.0.0": true}"#,
        )
        .unwrap();
        let old: Vec<String> = old_extension_versions(d.path())
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(old, vec!["a.b-1.10.0"]);
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
        assert_eq!(found[0].catalog_id.as_deref(), Some("p-rust"));
    }

    #[test]
    fn finds_extended_project_artifacts() {
        let d = tempfile::tempdir().unwrap();
        let web = d.path().join("web");
        fs::create_dir_all(web.join(".next")).unwrap();
        fs::create_dir_all(web.join(".turbo")).unwrap();
        fs::write(web.join("package.json"), "{}").unwrap();

        let py = d.path().join("py");
        fs::create_dir_all(py.join(".pytest_cache")).unwrap();
        fs::create_dir_all(py.join(".ruff_cache")).unwrap();
        fs::write(py.join("setup.py"), "").unwrap();

        let cpp = d.path().join("cpp");
        fs::create_dir_all(cpp.join("cmake-build-debug")).unwrap();
        fs::write(cpp.join("CMakeLists.txt"), "").unwrap();

        let tf = d.path().join("infra");
        fs::create_dir_all(tf.join(".terraform")).unwrap();
        fs::write(tf.join("main.tf"), "").unwrap();

        let unity = d.path().join("game");
        fs::create_dir_all(unity.join("Assets")).unwrap();
        fs::create_dir_all(unity.join("ProjectSettings")).unwrap();
        fs::create_dir_all(unity.join("Library")).unwrap();

        let found = local_targets(d.path(), 4);
        assert_eq!(found.len(), 7);
        for tg in &found {
            assert!(tg.catalog_id.is_some());
        }
    }

    #[test]
    fn maps_jetbrains_products() {
        assert_eq!(jetbrains_app("IntelliJIdea2026.2"), Some("idea"));
        assert_eq!(jetbrains_app("AndroidStudio2024.1"), Some("studio"));
        assert_eq!(jetbrains_app("Fleet"), Some("fleet"));
        assert_eq!(jetbrains_app("Daemon"), None);
    }
}
