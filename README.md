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
| Linguagem | Cargo, npm, pnpm, Yarn, Bun, Deno, pip, Poetry, uv, Maven, Gradle (caches e wrapper), Go, NuGet, Dart/Flutter, Composer, RubyGems |
| Ferramenta | Homebrew, Xcode DerivedData/DeviceSupport, cache do Android SDK, Docker buildx |
| IDE/Editor | VS Code, VS Code Insiders, Cursor e VSCodium (extensões, `CachedData`, `logs`, `Cache`...), JetBrains (um item por produto/versão), Android Studio, Sublime Text, Zed |
| Projeto Local | `target`, `node_modules`, `build`, `.gradle`, `.venv`/`venv`, `bin`/`obj`, `.dart_tool`, `vendor` (PHP), `.build` (Swift), detectados por marcadores como `Cargo.toml` e `package.json` |

Detalhes:
- A pasta de **extensões** de cada editor é marcada como **crítica** (exige confirmação reforçada).
- **Versões antigas de extensões** aparecem como itens separados e não críticos. A versão mais recente de cada extensão é mantida.
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
| `--lang pt\|en` | Idioma |

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
- Imagens e volumes do Docker não são limpos, só o cache do buildx. Use `docker system prune`.
- Na lixeira, o espaço só é liberado quando você esvaziá-la.
