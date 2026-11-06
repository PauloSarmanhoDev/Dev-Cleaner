# dev-cleaner

Ferramenta em Rust que varre e limpa caches, extensões e artefatos de build deixados por **linguagens/gerenciadores de pacotes**, **IDEs/editores**, **ferramentas** e **projetos locais**. Funciona em Windows, macOS e Linux e tem **duas interfaces**:

- **CLI** (terminal): `dev-cleaner`
- **GUI** (janela, Tauri 2): `dev-cleaner-gui`

As duas usam o mesmo núcleo (`dev-cleaner-core`). Detectam e limpam as mesmas coisas, com as mesmas regras de segurança. Interface em **português e inglês**.

## Estrutura

```
Cargo.toml              # workspace
crates/
├── core/               # lógica sem UI
│   └── src/{model,discovery,scanner,cleaner,safety,procs,config,store,pipeline,i18n}.rs
└── cli/                # interface de terminal
gui/
├── ui/                 # frontend (HTML/CSS/JS puro, sem Node)
└── src-tauri/          # backend Tauri
.github/workflows/      # CI e releases
```

## O que é detectado

| Tipo | Itens |
|---|---|
| Linguagem | Cargo/Rustup, npm, pnpm, Yarn, Bun, Deno, node-gyp/nvm, pip/pipenv/pdm, Poetry, uv, Conda/Mamba, Maven, Gradle, sbt/Coursier/Ivy, Kotlin Native, Go, NuGet, Dart/Flutter, Composer, RubyGems/Bundler, Elixir/Erlang (Hex, Mix, Rebar3), Haskell (Cabal, Stack, GHCup), Zig, Julia, Nim, C/C++ (Conan, vcpkg), LuaRocks, cpanm |
| Ferramenta | Homebrew, Xcode (DerivedData, DeviceSupport, CoreSimulator, Archives), CocoaPods, SwiftPM, Android SDK e **emuladores (AVD)**, Expo, Docker (buildx, **imagens e volumes sem uso**), Playwright/Puppeteer/Cypress, Electron, ccache/sccache/Bazel, pre-commit/Turborepo/Nx, Terraform/kubectl/Helm/AWS CLI, Vagrant/Minikube, **IA/ML** (Hugging Face, Ollama, PyTorch, Keras, LM Studio) |
| IDE/Editor | VS Code, VS Code Insiders, Cursor, Windsurf, Trae e VSCodium (extensões, `.obsolete`, `CachedData`, `logs`, `Cache`...), JetBrains (IDEA, WebStorm, PyCharm, RustRover, Fleet...), Android Studio, Visual Studio, Eclipse, Neovim, Sublime Text, Zed |
| Projeto Local | Rust (`target`), Node/Web (`node_modules`, `.next`, `.nuxt`, `.angular`, `.svelte-kit`, `.astro`, `.output`, `.turbo`, `.parcel-cache`), Maven/Gradle/sbt/Clojure, Python (`.venv`/`venv`, `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `.nox`), Dart/Flutter, PHP (`vendor`), Ruby (`vendor/bundle`), Swift/CocoaPods (`.build`, `Pods`), Elixir/Erlang (`_build`, `deps`), Haskell (`.stack-work`, `dist-newstyle`), Zig (`zig-out`, `.zig-cache`), CMake (`build`, `cmake-build-*`), .NET (`bin`/`obj`), Terraform (`.terraform`), Unity (`Library`, `Temp`, `Obj`, `Logs`), Unreal (`Intermediate`, `Binaries`, `DerivedDataCache`) |

Detalhes:
- A pasta de **extensões** de cada editor é marcada como **crítica** (exige confirmação reforçada).
- **Versões antigas de extensões** aparecem como itens separados e não críticos. O arquivo `.obsolete` do VS Code (extensões marcadas para remoção) tem prioridade; nas demais, a versão mais recente de cada extensão é mantida.
- Projetos locais cujo artefato é **rastreado pelo git** são ignorados, com aviso.

## Segurança

- **Lixeira por padrão.** Os itens são enviados para a lixeira do sistema e dá para recuperá-los. A remoção definitiva exige `--permanent` (CLI) ou a opção "Apagar permanentemente" (GUI).
- **Caminhos protegidos.** Raiz, home e pastas-base do usuário nunca são removidas, mesmo se selecionadas.
- **IDE aberta.** O programa detecta se a IDE dona do cache está em execução (🔒), avisa e pede confirmação para continuar.
- **Itens críticos** exigem digitar `APAGAR` (ou `DELETE`). A GUI e a CLI pedem isso, e o backend valida de novo.
- A GUI só envia **índices** do último resultado ao backend, nunca caminhos.
- Nenhum item vem marcado por padrão.
- **Auditoria.** Cada remoção é registrada em `audit.log` (Windows: `%LOCALAPPDATA%\dev-cleaner\`, Linux: `~/.local/share/dev-cleaner/`, macOS: `~/Library/Application Support/dev-cleaner/`).
- Arquivos bloqueados ou sem permissão viram avisos. Arquivos somente leitura são liberados e removidos.

## Requisitos

- [Rust](https://rustup.rs) estável recente (edition 2024).
- **Só para a GUI**: Windows precisa do WebView2 (já vem no Windows 10/11). Linux precisa do WebKitGTK ([pré-requisitos do Tauri](https://tauri.app/start/prerequisites/)). macOS não precisa de nada além do Xcode CLT.

## CLI

`cargo build` e `cargo run` na raiz compilam só o núcleo e a CLI.

```sh
cargo build --release          # target/release/dev-cleaner(.exe)

cargo run --release -- --dry-run                  # só lista
cargo run --release -- --root E:\Git              # + projetos locais
cargo run --release -- --root E:\Git --older-than 30d    # só o que não é usado há 30 dias
```

| Opção | Descrição |
|---|---|
| `-r, --root <DIR>` | Onde procurar projetos locais (padrão: `.`) |
| `-d, --depth <N>` | Profundidade máxima (padrão: 6) |
| `--no-lang` / `--no-ide` / `--no-local` | Exclui uma categoria |
| `--older-than <IDADE>` | Só itens sem uso há esse tempo (`30d`, `2w`, `12h`) |
| `--permanent` | Apaga definitivamente em vez de usar a lixeira |
| `-j, --jobs <N>` | Remoções em paralelo (padrão: 4) |
| `--dry-run` | Só lista, nunca remove |
| `--cached` | Reutiliza o último resultado, sem varrer de novo |
| `--json` | Imprime a lista em JSON |
| `-y, --yes` | Não pergunta. Exige `--select` ou `--all` |
| `--select <TEXTO>` | Com `--yes`: itens cujo nome/caminho contém o texto (repetível) |
| `--all` | Com `--yes`: todos os itens listados |
| `--include-critical` | Com `--yes`: permite itens críticos |
| `--force-running` | Com `--yes`: remove mesmo com a IDE aberta |
| `--config <ARQ>` | Arquivo de configuração |
| `--lang pt\|en` | Idioma (também localiza o `--help`) |
| `--list-supported` | Lista as linguagens, IDEs e ferramentas suportadas e sai (com `--json`, em JSON) |

Modo interativo: setas navegam, espaço marca, Enter confirma. **Ctrl+C** cancela a varredura. Um segundo Ctrl+C encerra o programa.

Exemplo para scripts e CI:
```sh
dev-cleaner --root . --no-lang --no-ide --yes --select node_modules --older-than 14d
```

## GUI

```sh
cargo run -p dev-cleaner-gui                # desenvolvimento
cargo build -p dev-cleaner-gui --release    # target/release/dev-cleaner-gui(.exe)
```

- Seletor de pasta, filtro por idade, opção de remoção permanente.
- Barra de progresso com bytes e arquivos lidos, e botão **Cancelar**.
- Tabela ordenável (clique no cabeçalho), filtro por tipo e gráfico de uso por categoria.
- 📂 abre o item no gerenciador de arquivos. 🔒 indica IDE aberta.
- Botão "Último resultado" carrega a varredura anterior sem refazê-la.
- Idioma (PT/EN) e tema claro/escuro.
- **Suportados**: lista tudo o que o programa detecta.
- **Configurações**: edita `config.toml` (idioma, filtro de idade, lixeira, exclusões e alvos extras). Salvar reescreve o arquivo e perde comentários feitos à mão.
- A barra de progresso mostra o arquivo/pasta sendo lido.

Instaladores: `cargo install tauri-cli --version "^2"`, depois `cargo tauri build` em `gui/src-tauri`. Antes, rode `cargo tauri icon gui/src-tauri/icons/icon.png --output gui/src-tauri/icons` para gerar todos os tamanhos de ícone.

## Configuração (opcional)

`<config>/dev-cleaner/config.toml` (Windows: `%APPDATA%`, Linux: `~/.config`, macOS: `~/Library/Application Support`):

```toml
language = "pt"        # pt | en
older_than = "30d"     # filtro padrão
permanent = false      # true = não usar a lixeira
exclude = ["~/.cargo/registry", "D:/importante"]

[[extra]]
name = "Meu cache"
path = "~/.meu-cache"
category = "tool"      # language | tool | ide | local
critical = false
```

## Desenvolvimento

```sh
cargo test --workspace --exclude dev-cleaner-gui
cargo clippy --workspace --exclude dev-cleaner-gui --all-targets -- -D warnings
cargo fmt --all
```

- **CI** (`.github/workflows/ci.yml`): fmt, clippy e testes em Linux, Windows e macOS, mais o build da GUI.
- **Release** (`release.yml`): ao criar uma tag `v*`, publica os binários da CLI e os instaladores da GUI (rascunho de release).

## Limitações conhecidas

- A detecção de IDE aberta compara o nome do processo. Em alguns sistemas ela pode não reconhecer a IDE.
- Imagens e volumes do Docker são removidos pelo próprio Docker (`image prune -a`, `volume prune -a`): não passam pela lixeira, mesmo no modo padrão. Volumes são itens críticos. Exige o `docker` no PATH e o daemon ativo.
- Ao apagar um AVD inteiro, o arquivo `<nome>.ini` ao lado da pasta `.avd` permanece; remova-o no AVD Manager se ele aparecer quebrado.
- As mensagens de erro do próprio clap (argumento inválido) continuam em inglês.
- Na lixeira, o espaço só é liberado quando você esvaziá-la.
