# dev-cleaner

Ferramenta em Rust que varre e limpa caches, extensões e artefatos de build deixados por **linguagens/gerenciadores de pacotes**, **IDEs/editores** e **projetos locais**. Funciona em Windows, macOS e Linux e tem **duas interfaces**:

- **CLI** (terminal interativo): `dev-cleaner`
- **GUI** (janela, Tauri 2): `dev-cleaner-gui`

Ambas usam o mesmo núcleo (`dev-cleaner-core`), então detectam e limpam exatamente as mesmas coisas.

## Estrutura

```
Cargo.toml              # workspace
crates/
├── core/               # lógica sem UI (descoberta, varredura, limpeza)
│   └── src/{model,discovery,scanner,cleaner}.rs
└── cli/                # interface de terminal
    └── src/{main,ui}.rs
gui/
├── ui/                 # frontend (HTML/CSS/JS puro, sem Node nem bundler)
└── src-tauri/          # backend Tauri (comandos `scan` e `clean`)
```

## Como funciona

1. **Descoberta** (`discovery.rs`): monta a lista de diretórios candidatos com a crate `dirs`. Só entram diretórios que existem.
2. **Varredura paralela** (`scanner.rs`): calcula tamanho e número de arquivos com `rayon` e `walkdir`. O progresso é reportado por callback (barra `indicatif` na CLI, evento `scan-progress` na GUI).
3. **Seleção**: tabela com `comfy-table` e menu `dialoguer` na CLI. Na GUI há uma tabela com checkboxes, filtro por tipo e totais.
4. **Confirmação**: itens críticos (extensões de IDE) exigem digitar `APAGAR`. A CLI e a GUI pedem isso, e o backend da GUI valida de novo.
5. **Limpeza** (`cleaner.rs`): remove arquivo por arquivo, em paralelo. Arquivos bloqueados ou sem permissão são pulados e listados como aviso. O programa não é interrompido.

### O que é detectado

| Tipo | Itens |
|---|---|
| Linguagem | Cargo (registry, git), npm, pnpm, Yarn, pip, Poetry, uv, Maven, Gradle, Go (módulos e build cache), NuGet |
| IDE/Editor | VS Code (`~/.vscode/extensions` **crítico**, `CachedData`, `logs`, `Cache`, `CachedExtensionVSIXs`, `Code Cache`), JetBrains (um item por produto/versão), Android Studio |
| Projeto Local | `target` (Rust, Maven), `node_modules`, `build`, `.gradle`, `.venv`/`venv`, `bin`/`obj` (.NET), detectados por `Cargo.toml`, `package.json`, `pom.xml`, `build.gradle(.kts)`, `pyproject.toml`, `requirements.txt`, `.csproj`, `.fsproj` ou `.sln` |

## Requisitos

- [Rust](https://rustup.rs) 1.85 ou mais recente (edition 2024).
- **Só para a GUI**:
  - Windows: WebView2 (já vem no Windows 10/11 atualizado).
  - Linux: pacotes de desenvolvimento do WebKitGTK (veja os [pré-requisitos do Tauri](https://tauri.app/start/prerequisites/)).
  - macOS: nada além do Xcode Command Line Tools.

## CLI

`cargo build` e `cargo run` na raiz usam só o núcleo e a CLI. O Tauri não é compilado.

```sh
cargo build --release          # binário em target/release/dev-cleaner(.exe)

cargo run --release -- --dry-run            # só lista (recomendado na primeira vez)
cargo run --release -- --root E:\Git        # inclui projetos locais
```

| Opção | Descrição |
|---|---|
| `-r, --root <DIR>` | Onde procurar projetos locais (padrão: `.`) |
| `-d, --depth <N>` | Profundidade máxima da busca (padrão: 6) |
| `--no-lang` / `--no-ide` / `--no-local` | Exclui uma categoria |
| `--dry-run` | Só lista, nunca remove |

No menu: setas navegam, espaço marca, Enter confirma. Rode dentro de um terminal. Um duplo clique no `.exe` abre e fecha a janela.

## GUI (Tauri)

```sh
cargo run -p dev-cleaner-gui                # desenvolvimento
cargo build -p dev-cleaner-gui --release    # binário em target/release/dev-cleaner-gui(.exe)
```

Na janela:
1. Marque Linguagens e IDEs, e opcionalmente informe uma **pasta de projetos** (vazio = não procura projetos locais).
2. Clique em **Analisar** e acompanhe a barra de progresso.
3. Filtre por tipo, marque os itens e clique em **Remover selecionados**.
4. Confirme. Se houver item crítico, digite `APAGAR`.

Instaladores (MSI, NSIS, dmg, deb) exigem a CLI do Tauri: `cargo install tauri-cli --version "^2"` e depois `cargo tauri build` dentro de `gui/src-tauri`.

> O frontend é HTML/JS estático em `gui/ui` (sem `npm`). Para trocar por React, Vue etc., ajuste `build.frontendDist` em `gui/src-tauri/tauri.conf.json`.

## Segurança

- **A remoção é permanente** (não usa a lixeira). Comece com `--dry-run` ou apenas analisando na GUI.
- Nenhum item vem marcado por padrão.
- A GUI só envia **índices** do último resultado ao backend, nunca caminhos. O frontend não consegue pedir a remoção de um caminho arbitrário.
- Extensões de IDE exigem a confirmação extra `APAGAR`. Depois de apagá-las, será preciso reinstalá-las.
- Feche as IDEs antes de limpar. Arquivos em uso são ignorados e aparecem como aviso.
- A busca de projetos ignora `.git` e pastas de artefatos conhecidas. Ela não segue links simbólicos.
