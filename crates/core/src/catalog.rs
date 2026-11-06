//! Lista do que o programa sabe detectar (para `--list-supported` na CLI e a tela da GUI)
//! e descrições detalhadas sobre o que cada item faz e as consequências de apagá-lo.
//! Mantenha em sincronia com `discovery.rs`.

use crate::i18n::{Lang, lang};
use crate::model::{Category, Target};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CatalogEntry {
    pub category: Category,
    pub name: &'static str,
    /// O que é limpo (texto neutro: caminhos e nomes de pastas).
    pub detail: &'static str,
    /// Explicação sobre o que é e as consequências de apagar (no idioma atual).
    pub info: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn e(
    category: Category,
    name: &'static str,
    detail: &'static str,
    info_pt: &'static str,
    info_en: &'static str,
) -> CatalogEntry {
    CatalogEntry {
        category,
        name,
        detail,
        info: if lang() == Lang::Pt { info_pt } else { info_en },
    }
}

pub fn catalog() -> Vec<CatalogEntry> {
    use Category::{Ide as I, Language as L, LocalProject as P, Tool as T};
    vec![
        // Linguagens e gerenciadores de pacotes
        e(
            L,
            "Rust (Cargo / Rustup)",
            "~/.cargo/registry, ~/.cargo/git, ~/.rustup/downloads, ~/.rustup/toolchains (critical)",
            "O que é: caches de crates e repositórios git baixados pelo Cargo, mais instaladores e toolchains do Rustup.\n\nConsequência ao apagar:\n• Cargo registry/git e Rustup downloads: seguro. O Cargo baixará de novo as crates na próxima compilação.\n• Rustup toolchains (⚠ crítico): remove os compiladores Rust instalados (stable, nightly, targets); será preciso rodar `rustup toolchain install` novamente.",
            "What it is: crate and git caches downloaded by Cargo, plus Rustup installers and toolchains.\n\nConsequence of deleting:\n• Cargo registry/git and Rustup downloads: safe. Cargo will re-download crates on the next build.\n• Rustup toolchains (⚠ critical): removes installed Rust compilers (stable, nightly, targets); you will need to run `rustup toolchain install` again.",
        ),
        e(
            L,
            "npm",
            "~/.npm/_cacache",
            "O que é: cache de pacotes tarballs e metadados HTTP baixados pelo npm.\n\nConsequência ao apagar: seguro. Projetos existentes continuam funcionando; novos `npm install` baixarão os pacotes da internet novamente.",
            "What it is: cache of package tarballs and HTTP metadata downloaded by npm.\n\nConsequence of deleting: safe. Existing projects keep working; future `npm install` runs will re-download packages from the internet.",
        ),
        e(
            L,
            "pnpm",
            "store",
            "O que é: armazenamento endereçado por conteúdo (store) onde o pnpm guarda uma única cópia dos pacotes e cria hardlinks para os projetos.\n\nConsequência ao apagar: seguro para o código, mas se você remover pacotes que ainda estão linkados, pode ser necessário rodar `pnpm install` nos projetos ativos.",
            "What it is: content-addressable store where pnpm keeps a single copy of packages and hardlinks them into projects.\n\nConsequence of deleting: safe for source code, though active projects may need a fresh `pnpm install` if their unreferenced store entries are removed.",
        ),
        e(
            L,
            "Yarn",
            "cache",
            "O que é: cache global de pacotes baixados pelo Yarn (v1 e Berry).\n\nConsequência ao apagar: seguro. Na próxima instalação de dependências o Yarn fará o download novamente.",
            "What it is: global package cache downloaded by Yarn (v1 and Berry).\n\nConsequence of deleting: safe. Yarn will re-download packages on the next install.",
        ),
        e(
            L,
            "Bun",
            "~/.bun/install/cache",
            "O que é: cache global de pacotes npm baixados pelo gerenciador do Bun.\n\nConsequência ao apagar: seguro. O Bun baixará os pacotes de novo quando necessário (`bun install`).",
            "What it is: global cache of npm packages downloaded by Bun's package manager.\n\nConsequence of deleting: safe. Bun will re-download packages when needed (`bun install`).",
        ),
        e(
            L,
            "Deno",
            "cache",
            "O que é: módulos remotos (TypeScript/JavaScript/npm) e artefatos compilados em cache pelo Deno.\n\nConsequência ao apagar: seguro. O Deno baixará e recompilará os módulos automaticamente na próxima execução.",
            "What it is: remote modules (TypeScript/JavaScript/npm) and compiled artifacts cached by Deno.\n\nConsequence of deleting: safe. Deno will automatically re-fetch and recompile modules on the next run.",
        ),
        e(
            L,
            "node-gyp / nvm",
            "node-gyp cache, ~/.nvm/.cache",
            "O que é: cabeçalhos do Node.js usados pelo `node-gyp` para compilar addons nativos C++ e arquivos tarballs do instalador `nvm`.\n\nConsequência ao apagar: seguro. Serão baixados novamente apenas se você compilar um addon nativo ou instalar outra versão do Node via nvm.",
            "What it is: Node.js headers used by `node-gyp` to compile native C++ addons, and installer tarballs cached by `nvm`.\n\nConsequence of deleting: safe. They are only re-downloaded when building a native addon or installing a new Node version via nvm.",
        ),
        e(
            L,
            "pip / pipenv / pdm",
            "cache",
            "O que é: arquivos `.whl` (wheels) e pacotes-fonte baixados pelo pip, Pipenv e PDM.\n\nConsequência ao apagar: seguro. Ambientes virtuais já criados não são afetados; futuras instalações baixarão do PyPI novamente.",
            "What it is: `.whl` (wheel) files and source distributions cached by pip, Pipenv and PDM.\n\nConsequence of deleting: safe. Existing virtualenvs are unaffected; future installs will re-download from PyPI.",
        ),
        e(
            L,
            "Poetry",
            "cache",
            "O que é: cache de pacotes e repositórios do gerenciador Python Poetry (ATENÇÃO: em alguns sistemas pode conter virtualenvs se configurado junto ao cache).\n\nConsequência ao apagar: pacotes serão baixados de novo; se seus virtualenvs ficavam dentro da pasta de cache, baste rodar `poetry install`.",
            "What it is: package and repository cache for Python Poetry.\n\nConsequence of deleting: packages will be re-downloaded; if your virtualenvs were stored inside the cache directory, re-run `poetry install`.",
        ),
        e(
            L,
            "uv",
            "cache",
            "O que é: cache global de wheels e pacotes compilados pelo `uv` (Astral).\n\nConsequência ao apagar: seguro. O `uv` recriará o cache sob demanda; ambientes `.venv` que usam hardlinks continuam funcionando.",
            "What it is: global wheel and built-package cache used by `uv` (Astral).\n\nConsequence of deleting: safe. `uv` rebuilds the cache on demand; existing `.venv` environments using hardlinks continue to work.",
        ),
        e(
            L,
            "Conda / Mamba",
            "~/.conda/pkgs, ~/miniconda3/pkgs, ~/anaconda3/pkgs, ~/miniforge3/pkgs",
            "O que é: arquivos compactados e descompactados de pacotes baixados pelo Conda/Mamba (`pkgs`). Costuma ocupar muitos gigabytes.\n\nConsequência ao apagar: cuidado se houver ambientes Conda ativos usando hardlinks diretos para `pkgs`; caso algum ambiente quebre, recrie-o ou prefira `conda clean --all`.",
            "What it is: downloaded and extracted package tarballs in Conda/Mamba (`pkgs`). Often takes several gigabytes.\n\nConsequence of deleting: be careful if active Conda environments hardlink into `pkgs`; if an environment breaks, recreate it or use `conda clean --all`.",
        ),
        e(
            L,
            "Maven",
            "~/.m2/repository",
            "O que é: repositório local de dependências `.jar` e plugins do Apache Maven.\n\nConsequência ao apagar: seguro, porém na próxima build o Maven baixará todas as dependências novamente (e artefatos instalados apenas localmente com `mvn install` precisarão ser recompilados).",
            "What it is: local repository of `.jar` dependencies and plugins for Apache Maven.\n\nConsequence of deleting: safe, but Maven will re-download all dependencies on the next build (and locally installed `mvn install` artifacts will need to be rebuilt).",
        ),
        e(
            L,
            "Gradle",
            "~/.gradle/caches, ~/.gradle/wrapper/dists, ~/.gradle/daemon",
            "O que é: dependências baixadas, caches de compilação, versões do Gradle Wrapper (`dists`) e logs dos daemons.\n\nConsequência ao apagar: seguro. A próxima build de projetos Java/Kotlin/Android demorará mais pois baixará o Gradle e as bibliotecas novamente.",
            "What it is: downloaded dependencies, build caches, Gradle Wrapper distributions (`dists`) and daemon logs.\n\nConsequence of deleting: safe. The next Java/Kotlin/Android build will take longer while it re-downloads Gradle and dependencies.",
        ),
        e(
            L,
            "Scala / JVM (sbt, Coursier, Ivy)",
            "~/.ivy2/cache, ~/.sbt/boot, Coursier cache",
            "O que é: caches de dependências JVM/Scala gerenciados por Ivy2, sbt e Coursier.\n\nConsequência ao apagar: seguro. O sbt/Coursier fará o download das bibliotecas novamente na próxima compilação.",
            "What it is: JVM/Scala dependency caches managed by Ivy2, sbt and Coursier.\n\nConsequence of deleting: safe. sbt/Coursier will re-download libraries on the next build.",
        ),
        e(
            L,
            "Kotlin Native",
            "~/.konan",
            "O que é: compilador Kotlin/Native, toolchains LLVM e dependências nativas pré-baixadas.\n\nConsequência ao apagar: seguro. O Gradle baixará o toolchain Kotlin/Native novamente quando você compilar um projeto Kotlin Multiplatform/Native.",
            "What it is: Kotlin/Native compiler, LLVM toolchains and pre-downloaded native dependencies.\n\nConsequence of deleting: safe. Gradle will re-download the Kotlin/Native toolchain when building a Kotlin Multiplatform/Native project.",
        ),
        e(
            L,
            "Go",
            "~/go/pkg/mod, go-build",
            "O que é: módulos Go baixados (`go/pkg/mod`) e cache de compilação incremental (`go-build`).\n\nConsequência ao apagar: seguro. O Go baixará os módulos e recompilará os pacotes automaticamente no próximo `go build`/`go run`.",
            "What it is: downloaded Go modules (`go/pkg/mod`) and incremental build cache (`go-build`).\n\nConsequence of deleting: safe. Go will re-fetch modules and recompile packages automatically on the next `go build`/`go run`.",
        ),
        e(
            L,
            ".NET (NuGet)",
            "~/.nuget/packages, NuGet v3-cache",
            "O que é: cache global de pacotes NuGet (`.nupkg`/DLLs) e cache HTTP v3 do .NET.\n\nConsequência ao apagar: seguro. O `dotnet restore` ou Visual Studio baixará os pacotes novamente na próxima compilação.",
            "What it is: global NuGet package cache (`.nupkg`/DLLs) and .NET HTTP v3 cache.\n\nConsequence of deleting: safe. `dotnet restore` or Visual Studio will re-download packages on the next build.",
        ),
        e(
            L,
            "Dart / Flutter",
            "~/.pub-cache",
            "O que é: pacotes baixados do pub.dev pelo Dart e Flutter.\n\nConsequência ao apagar: seguro. Rode `flutter pub get` ou `dart pub get` nos seus projetos para baixar as dependências novamente.",
            "What it is: packages downloaded from pub.dev by Dart and Flutter.\n\nConsequence of deleting: safe. Run `flutter pub get` or `dart pub get` in your projects to re-download dependencies.",
        ),
        e(
            L,
            "PHP (Composer)",
            "cache",
            "O que é: arquivos zip e metadados de pacotes PHP baixados pelo Composer.\n\nConsequência ao apagar: seguro. Pastas `vendor/` dos projetos não são afetadas; novos `composer install` baixarão da internet.",
            "What it is: zip archives and package metadata cached by PHP Composer.\n\nConsequence of deleting: safe. Project `vendor/` folders are unaffected; future `composer install` runs will re-download from the network.",
        ),
        e(
            L,
            "Ruby (RubyGems / Bundler)",
            "~/.gem, ~/.bundle/cache",
            "O que é: cache de arquivos `.gem` do RubyGems e do Bundler no diretório do usuário.\n\nConsequência ao apagar: seguro; se alguma gem de usuário estava instalada diretamente em `~/.gem`, basta rodar `bundle install` novamente.",
            "What it is: cached `.gem` files from RubyGems and Bundler in the user directory.\n\nConsequence of deleting: safe; if any user gem was installed directly under `~/.gem`, simply run `bundle install` again.",
        ),
        e(
            L,
            "Elixir / Erlang (Hex, Mix, Rebar3)",
            "~/.hex/packages, ~/.mix/archives, rebar3 cache",
            "O que é: pacotes Hex baixados, arquivos do Mix (ex.: gerador `phx_new`) e cache do Rebar3.\n\nConsequência ao apagar: seguro. `mix deps.get` baixará os pacotes novamente; arquivos do Mix podem ser reinstalados com `mix archive.install`.",
            "What it is: downloaded Hex packages, Mix archives (e.g. `phx_new` generator) and Rebar3 cache.\n\nConsequence of deleting: safe. `mix deps.get` will re-fetch packages; Mix archives can be reinstalled with `mix archive.install`.",
        ),
        e(
            L,
            "Haskell (Cabal, Stack, GHCup)",
            "~/.cabal/packages, ~/.cabal/store, ~/.stack, ~/.ghcup/cache",
            "O que é: pacotes compilados e fontes do Cabal, snapshots/GHC do Stack e instaladores em cache do GHCup.\n\nConsequência ao apagar: seguro para projetos, mas a recompilação de bibliotecas Haskell no próximo `cabal build` ou `stack build` pode ser demorada.",
            "What it is: compiled packages and sources from Cabal, Stack snapshots/GHC and cached GHCup installers.\n\nConsequence of deleting: safe for projects, though recompiling Haskell libraries on the next `cabal build` or `stack build` can take a while.",
        ),
        e(
            L,
            "Zig",
            "cache",
            "O que é: cache global de artefatos de compilação e pacotes baixados pelo compilador Zig.\n\nConsequência ao apagar: seguro. O Zig recompilará os artefatos automaticamente na próxima build.",
            "What it is: global cache of build artifacts and packages downloaded by the Zig compiler.\n\nConsequence of deleting: safe. Zig will automatically recompile artifacts on the next build.",
        ),
        e(
            L,
            "Julia",
            "~/.julia/compiled",
            "O que é: arquivos de pré-compilação (`.ji`/`.so`/`.dll`) dos pacotes Julia instalados.\n\nConsequência ao apagar: seguro. Os pacotes continuam instalados; a Julia apenas pré-compilará novamente na próxima vez que forem importados (`using`).",
            "What it is: precompiled cache files (`.ji`/`.so`/`.dll`) of installed Julia packages.\n\nConsequence of deleting: safe. Packages remain installed; Julia will simply precompile them again the next time they are imported (`using`).",
        ),
        e(
            L,
            "Nim (Nimble)",
            "~/.nimble/pkgs2",
            "O que é: pacotes instalados pelo gerenciador Nimble da linguagem Nim.\n\nConsequência ao apagar: será necessário rodar `nimble install` novamente nos seus projetos Nim.",
            "What it is: packages installed by Nim's Nimble package manager.\n\nConsequence of deleting: you will need to run `nimble install` again for your Nim projects.",
        ),
        e(
            L,
            "C / C++ (Conan, vcpkg)",
            "~/.conan2/p, ~/.conan/data, vcpkg/archives",
            "O que é: pacotes C/C++ baixados/compilados pelo Conan e arquivos binários em cache do vcpkg (`archives`).\n\nConsequência ao apagar: seguro. Na próxima configuração/build do projeto, o Conan ou vcpkg baixará/recompilará as bibliotecas.",
            "What it is: C/C++ packages downloaded/built by Conan and cached binary archives from vcpkg (`archives`).\n\nConsequence of deleting: safe. Conan or vcpkg will re-download/rebuild libraries on the next project configure/build.",
        ),
        e(
            L,
            "Lua (LuaRocks)",
            "cache",
            "O que é: pacotes (rocks) e manifestos baixados em cache pelo LuaRocks.\n\nConsequência ao apagar: seguro. Rocks já instalados continuam funcionando.",
            "What it is: rocks and manifests cached by LuaRocks.\n\nConsequence of deleting: safe. Already-installed rocks continue to work.",
        ),
        e(
            L,
            "Perl (cpanm)",
            "~/.cpanm/work",
            "O que é: diretórios temporários de extração e compilação de módulos Perl do `cpanminus`.\n\nConsequência ao apagar: totalmente seguro. São apenas sobras de instalações anteriores.",
            "What it is: temporary extraction and build directories left by Perl's `cpanminus`.\n\nConsequence of deleting: completely safe. These are just leftovers from previous installations.",
        ),
        // Ferramentas
        e(
            T,
            "Homebrew",
            "cache",
            "O que é: instaladores e garrafas (bottles/DMGs) baixados pelo Homebrew (macOS/Linux).\n\nConsequência ao apagar: totalmente seguro (equivalente a `brew cleanup`). Os programas instalados não são afetados.",
            "What it is: installers and bottles/DMGs downloaded by Homebrew (macOS/Linux).\n\nConsequence of deleting: completely safe (equivalent to `brew cleanup`). Installed programs are unaffected.",
        ),
        e(
            T,
            "Xcode",
            "DerivedData, iOS/watchOS DeviceSupport, CoreSimulator Caches, Archives (critical)",
            "O que é: artefatos intermediários de build (`DerivedData`), símbolos de depuração de aparelhos (`DeviceSupport`), caches do simulador e pacotes de release (`Archives`).\n\nConsequência ao apagar:\n• DerivedData / DeviceSupport / CoreSimulator Caches: seguro; o Xcode recria quando necessário.\n• Archives (⚠ crítico): apaga builds `.xcarchive` antigas geradas para envio à App Store / símbolo de crash.",
            "What it is: intermediate build artifacts (`DerivedData`), device debug symbols (`DeviceSupport`), simulator caches and release bundles (`Archives`).\n\nConsequence of deleting:\n• DerivedData / DeviceSupport / CoreSimulator Caches: safe; Xcode recreates them as needed.\n• Archives (⚠ critical): permanently removes `.xcarchive` builds generated for App Store submission / crash symbolication.",
        ),
        e(
            T,
            "CocoaPods / SwiftPM",
            "~/Library/Caches/CocoaPods, ~/.cocoapods/repos, org.swift.swiftpm",
            "O que é: cache de pods e repositórios de specs do CocoaPods, e cache de pacotes do Swift Package Manager.\n\nConsequência ao apagar: seguro. Serão baixados novamente no próximo `pod install` ou build no Xcode/Swift.",
            "What it is: cached pods and spec repos for CocoaPods, plus Swift Package Manager package cache.\n\nConsequence of deleting: safe. They will be re-downloaded on the next `pod install` or Xcode/Swift build.",
        ),
        e(
            T,
            "Android SDK",
            "~/.android/cache, ~/.android/build-cache",
            "O que é: caches temporários do Android SDK e do Android Gradle Plugin.\n\nConsequência ao apagar: totalmente seguro. O Android Studio/Gradle recria os arquivos automaticamente.",
            "What it is: temporary caches for the Android SDK and Android Gradle Plugin.\n\nConsequence of deleting: completely safe. Android Studio/Gradle recreates them automatically.",
        ),
        e(
            T,
            "Android emulators (AVD)",
            "~/.android/avd: snapshots; whole AVD (critical)",
            "O que é: emuladores Android criados no AVD Manager.\n\nConsequência ao apagar:\n• Snapshots: seguro. O emulador permanece intacto, mas na próxima vez fará um Cold Boot (inicialização completa).\n• AVD inteiro (⚠ crítico): apaga o aparelho virtual inteiro (e o `.ini` correspondente), incluindo apps e dados salvos nele.",
            "What it is: Android Virtual Devices created in AVD Manager.\n\nConsequence of deleting:\n• Snapshots: safe. The emulator remains intact, but its next launch will be a Cold Boot.\n• Whole AVD (⚠ critical): deletes the entire virtual device (and its sibling `.ini` file), including all apps and data inside it.",
        ),
        e(
            T,
            "Expo",
            "~/.expo",
            "O que é: cache de templates, pacotes e APKs/IPAs de desenvolvimento do Expo (React Native).\n\nConsequência ao apagar: seguro. O Expo recriará os caches na próxima execução.",
            "What it is: template, package and development client cache for Expo (React Native).\n\nConsequence of deleting: safe. Expo will recreate the caches on the next run.",
        ),
        e(
            T,
            "Docker buildx",
            "~/.docker/buildx",
            "O que é: estado local e metadados de builders do Docker Buildx.\n\nConsequência ao apagar: seguro. O Docker recria a pasta no próximo `docker buildx build`.",
            "What it is: local state and builder metadata for Docker Buildx.\n\nConsequence of deleting: safe. Docker recreates the folder on the next `docker buildx build`.",
        ),
        e(
            T,
            "Docker images",
            "docker image prune -a (unused images)",
            "O que é: imagens Docker baixadas ou construídas que não estão associadas a nenhum container existente.\n\nConsequência ao apagar: executa `docker image prune -a -f` (NÃO vai para a lixeira). Containers existentes não são afetados, mas imagens sem container precisarão de `docker pull`/`docker build` novamente.",
            "What it is: downloaded or built Docker images not used by any existing container.\n\nConsequence of deleting: runs `docker image prune -a -f` (bypasses the system trash). Existing containers are unaffected, but unused images will need to be pulled or built again.",
        ),
        e(
            T,
            "Docker volumes",
            "docker volume prune -a (unused volumes, critical)",
            "O que é: volumes locais do Docker que não estão montados em nenhum container no momento.\n\nConsequência ao apagar (⚠ crítico): executa `docker volume prune -a -f` (NÃO vai para a lixeira). Se você tiver bancos de dados locais (Postgres, MySQL, Redis...) cujo container foi removido, os dados do volume serão perdidos.",
            "What it is: local Docker volumes not currently mounted by any container.\n\nConsequence of deleting (⚠ critical): runs `docker volume prune -a -f` (bypasses the system trash). If you have local databases (Postgres, MySQL, Redis...) whose container was removed, the data in those volumes will be permanently lost.",
        ),
        e(
            T,
            "Playwright / Puppeteer / Cypress",
            "ms-playwright, ~/.cache/puppeteer, Cypress cache",
            "O que é: binários completos de navegadores (Chromium, Firefox, WebKit) e do Cypress baixados para testes automatizados E2E.\n\nConsequência ao apagar: seguro. Se voltar a rodar os testes, será preciso rodar `npx playwright install`, `puppeteer browsers install` ou `cypress install`.",
            "What it is: full browser binaries (Chromium, Firefox, WebKit) and Cypress binaries downloaded for E2E testing.\n\nConsequence of deleting: safe. Before running tests again, you will need `npx playwright install`, `puppeteer browsers install` or `cypress install`.",
        ),
        e(
            T,
            "Electron",
            "electron cache, electron-builder cache",
            "O que é: binários pré-compilados do Electron, NSIS/WiX e ferramentas de empacotamento do `electron-builder`.\n\nConsequência ao apagar: seguro. Serão baixados novamente quando você rodar ou empacotar um app Electron.",
            "What it is: prebuilt Electron binaries, NSIS/WiX and packaging tools cached by `electron-builder`.\n\nConsequence of deleting: safe. They will be re-downloaded when running or packaging an Electron app.",
        ),
        e(
            T,
            "ccache / sccache / Bazel",
            "ccache, sccache, bazel cache",
            "O que é: caches de compilação de C/C++/Rust (`ccache`, `sccache`) e do sistema de build `Bazel`.\n\nConsequência ao apagar: totalmente seguro. Apenas fará com que a próxima compilação seja feita do zero (mais lenta).",
            "What it is: compiler caches for C/C++/Rust (`ccache`, `sccache`) and the `Bazel` build system.\n\nConsequence of deleting: completely safe. It only makes the next build compile from scratch (slower).",
        ),
        e(
            T,
            "pre-commit / Turborepo / Nx",
            "pre-commit, turbo, nx cache",
            "O que é: ambientes de hooks git do `pre-commit` e caches remotos/locais de monorepos (`Turborepo`, `Nx`).\n\nConsequência ao apagar: seguro. O `pre-commit` reinstalará os hooks no próximo commit e o Turbo/Nx recalculará as tasks.",
            "What it is: git hook environments for `pre-commit` and monorepo task caches (`Turborepo`, `Nx`).\n\nConsequence of deleting: safe. `pre-commit` will reinstall hook environments on the next commit and Turbo/Nx will recompute tasks.",
        ),
        e(
            T,
            "Terraform / Kubernetes / Helm / AWS",
            "~/.terraform.d/plugin-cache, ~/.kube/cache, helm cache, ~/.aws/cli/cache",
            "O que é: binários de providers Terraform em cache, cache de descoberta de API do `kubectl`, charts do Helm e tokens temporários do AWS CLI.\n\nConsequência ao apagar: seguro. As ferramentas recriam esses caches automaticamente.",
            "What it is: cached Terraform provider binaries, `kubectl` API discovery cache, Helm chart cache and temporary AWS CLI cache.\n\nConsequence of deleting: safe. These tools recreate their caches automatically.",
        ),
        e(
            T,
            "Vagrant / Minikube",
            "~/.vagrant.d/boxes (critical), ~/.minikube/cache",
            "O que é: imagens base de máquinas virtuais do Vagrant (`boxes`) e ISOs/binários em cache do Minikube.\n\nConsequência ao apagar:\n• Minikube cache: seguro.\n• Vagrant boxes (⚠ crítico): remove os templates de VM baixados; VMs já criadas continuam existindo, mas novas VMs precisarão baixar a box novamente.",
            "What it is: Vagrant base VM images (`boxes`) and cached Minikube ISOs/binaries.\n\nConsequence of deleting:\n• Minikube cache: safe.\n• Vagrant boxes (⚠ critical): removes downloaded VM box templates; existing VMs still work, but new VMs will need to re-download the box.",
        ),
        e(
            T,
            "AI / ML (Hugging Face, Ollama, PyTorch, Keras, LM Studio)",
            "huggingface, ~/.ollama/models, torch, ~/.keras, lm-studio models (critical)",
            "O que é: pesos de modelos de IA (LLMs, difusão, visão), checkpoints e datasets baixados por Hugging Face, Ollama, PyTorch, Keras e LM Studio.\n\nConsequência ao apagar (⚠ crítico): não estraga programas, mas os modelos costumam ter dezenas de gigabytes e precisarão ser baixados novamente (ex.: `ollama pull`).",
            "What it is: AI model weights (LLMs, diffusion, vision), checkpoints and datasets downloaded by Hugging Face, Ollama, PyTorch, Keras and LM Studio.\n\nConsequence of deleting (⚠ critical): does not break apps, but models are often tens of gigabytes and will need to be downloaded again (e.g. `ollama pull`).",
        ),
        // IDEs e editores
        e(
            I,
            "VS Code / Insiders",
            "extensions, old extension versions (.obsolete), CachedData, logs, Cache",
            "O que é: extensões instaladas, versões antigas/obsoletas de extensões e caches internos (V8 bytecode `CachedData`, `Cache`, `logs`, VSIXs baixados).\n\nConsequência ao apagar:\n• Caches, logs e extensões antigas (`old extension`): totalmente seguro. A versão mais recente de cada extensão é mantida.\n• Pasta `extensions` inteira (⚠ crítico): apaga TODAS as extensões instaladas no editor.",
            "What it is: installed extensions, old/obsolete extension versions and internal caches (V8 bytecode `CachedData`, `Cache`, `logs`, downloaded VSIXs).\n\nConsequence of deleting:\n• Caches, logs and old extensions (`old extension`): completely safe. The latest version of each extension is kept.\n• Full `extensions` folder (⚠ critical): deletes ALL installed extensions in the editor.",
        ),
        e(
            I,
            "Cursor / Windsurf / Trae / VSCodium",
            "extensions, old extension versions (.obsolete), CachedData, logs, Cache",
            "O que é: extensões e caches internos de editores derivados do VS Code (Cursor, Windsurf, Trae, VSCodium).\n\nConsequência ao apagar:\n• Caches, logs e extensões antigas: totalmente seguro.\n• Pasta `extensions` inteira (⚠ crítico): remove todas as extensões instaladas no editor.",
            "What it is: extensions and internal caches of VS Code-based editors (Cursor, Windsurf, Trae, VSCodium).\n\nConsequence of deleting:\n• Caches, logs and old extensions: completely safe.\n• Full `extensions` folder (⚠ critical): removes all installed extensions in the editor.",
        ),
        e(
            I,
            "JetBrains",
            "IntelliJ IDEA, WebStorm, PyCharm, CLion, GoLand, Rider, PhpStorm, RubyMine, DataGrip, RustRover, DataSpell, Aqua, Fleet",
            "O que é: índices de código, caches de análise e histórico local de cada IDE e versão da JetBrains (em `Caches/JetBrains`). Suas configurações e plugins ficam em outra pasta e NÃO são tocados.\n\nConsequência ao apagar: seguro (principalmente para versões antigas de IDEs que você já atualizou). Na versão atual, a IDE apenas reindexará os projetos ao abrir.",
            "What it is: code indexes, analysis caches and local history for each JetBrains IDE and version (under `Caches/JetBrains`). Your settings and plugins live in a separate folder and are NOT touched.\n\nConsequence of deleting: safe (especially for old IDE versions after upgrading). For the current version, the IDE will simply re-index projects on next launch.",
        ),
        e(
            I,
            "Android Studio",
            "caches",
            "O que é: índices e caches de análise do Android Studio (por versão).\n\nConsequência ao apagar: seguro. Versões antigas liberam bastante espaço; na versão em uso, o Android Studio reindexará o projeto na próxima abertura.",
            "What it is: code indexes and analysis caches for Android Studio (per version).\n\nConsequence of deleting: safe. Old versions free up significant space; for the current version, Android Studio will re-index on next launch.",
        ),
        e(
            I,
            "Visual Studio",
            "ComponentModelCache",
            "O que é: cache de composição MEF (`ComponentModelCache`) de extensões do Visual Studio.\n\nConsequência ao apagar: totalmente seguro (inclusive recomendado para corrigir falhas de extensões no Visual Studio). Ele é reconstruído na próxima abertura.",
            "What it is: MEF composition cache (`ComponentModelCache`) for Visual Studio extensions.\n\nConsequence of deleting: completely safe (often recommended to fix Visual Studio extension glitches). It is rebuilt on the next launch.",
        ),
        e(
            I,
            "Eclipse",
            "~/.p2/pool (critical)",
            "O que é: bundle pool compartilhado (`~/.p2/pool`) usado pelo instalador Eclipse Oomph para guardar os plugins e binários das instalações do Eclipse.\n\nConsequência ao apagar (⚠ crítico): se você ainda tem um Eclipse instalado via Oomph que referencia essa pasta, ele deixará de abrir. Só apague se já desinstalou o Eclipse.",
            "What it is: shared bundle pool (`~/.p2/pool`) used by the Eclipse Oomph installer to store plugins and binaries for Eclipse installations.\n\nConsequence of deleting (⚠ critical): if you still have an Oomph-installed Eclipse referencing this pool, it will stop launching. Only delete if you no longer use Eclipse.",
        ),
        e(
            I,
            "Neovim",
            "nvim cache, ~/.local/share/nvim / nvim-data (critical)",
            "O que é: cache temporário do Neovim e pasta de dados (`nvim-data` / `~/.local/share/nvim`, onde ficam plugins do Lazy/Packer e servidores LSP do Mason).\n\nConsequência ao apagar:\n• `Neovim cache`: totalmente seguro.\n• `Neovim data` (⚠ crítico): apaga plugins, árvores do Treesitter e LSPs baixados pelo Mason (sua configuração em `.config/nvim` é mantida, e os plugins são baixados de novo ao abrir).",
            "What it is: temporary Neovim cache and data folder (`nvim-data` / `~/.local/share/nvim`, where Lazy/Packer plugins and Mason LSP servers live).\n\nConsequence of deleting:\n• `Neovim cache`: completely safe.\n• `Neovim data` (⚠ critical): removes downloaded plugins, Treesitter parsers and Mason LSPs (your config in `.config/nvim` is kept, and plugins re-download on launch).",
        ),
        e(
            I,
            "Sublime Text",
            "cache",
            "O que é: índices de símbolos de projetos e caches de sintaxe do Sublime Text.\n\nConsequência ao apagar: totalmente seguro. Seus pacotes e configurações ficam em `Packages` e não são afetados.",
            "What it is: project symbol indexes and syntax caches for Sublime Text.\n\nConsequence of deleting: completely safe. Your installed packages and settings live in `Packages` and are unaffected.",
        ),
        e(
            I,
            "Zed",
            "cache, data",
            "O que é: cache e dados locais do editor Zed (incluindo servidores de linguagem LSP e extensões baixadas automaticamente).\n\nConsequência ao apagar: seguro. Suas configurações (`settings.json`) ficam em outra pasta; o Zed baixará novamente os LSPs necessários ao abrir um projeto.",
            "What it is: cache and local data for the Zed editor (including automatically downloaded language servers and extensions).\n\nConsequence of deleting: safe. Your settings (`settings.json`) are kept; Zed will re-download needed language servers when you open a project.",
        ),
        // Projetos locais
        e(
            P,
            "Rust",
            "target (Cargo.toml)",
            "O que é: pasta `target/` de um projeto Rust, contendo binários compilados e dependências intermediárias (costuma ter vários GB).\n\nConsequência ao apagar: totalmente seguro (equivalente a `cargo clean`). O próximo `cargo build`/`cargo run` recompilará o projeto.",
            "What it is: a Rust project's `target/` directory containing compiled binaries and intermediate dependencies (often several GB).\n\nConsequence of deleting: completely safe (equivalent to `cargo clean`). The next `cargo build`/`cargo run` will recompile the project.",
        ),
        e(
            P,
            "Node / Web (Next.js, Nuxt, Angular, SvelteKit, Astro, Turbo, Parcel)",
            "node_modules, .next, .nuxt, .angular, .svelte-kit, .astro, .output, .turbo, .parcel-cache (package.json)",
            "O que é: pacotes instalados (`node_modules`) e pastas de build/cache geradas por frameworks web (`.next`, `.nuxt`, `.angular`, `.svelte-kit`, `.astro`, `.output`, `.turbo`, `.parcel-cache`).\n\nConsequência ao apagar: seguro para o código-fonte. Para voltar a trabalhar no projeto, rode `npm/pnpm/yarn/bun install` (se apagou `node_modules`) ou `npm run dev`/`build`.",
            "What it is: installed packages (`node_modules`) and build/cache folders generated by web frameworks (`.next`, `.nuxt`, `.angular`, `.svelte-kit`, `.astro`, `.output`, `.turbo`, `.parcel-cache`).\n\nConsequence of deleting: safe for source code. To work on the project again, run `npm/pnpm/yarn/bun install` (if `node_modules` was removed) or `npm run dev`/`build`.",
        ),
        e(
            P,
            "Maven",
            "target (pom.xml)",
            "O que é: pasta `target/` com classes `.class` compiladas e pacotes `.jar`/`.war` gerados pelo Maven.\n\nConsequência ao apagar: totalmente seguro (equivalente a `mvn clean`).",
            "What it is: `target/` folder with compiled `.class` files and `.jar`/`.war` packages generated by Maven.\n\nConsequence of deleting: completely safe (equivalent to `mvn clean`).",
        ),
        e(
            P,
            "Gradle",
            "build, .gradle (build.gradle[.kts])",
            "O que é: saída de compilação (`build/`) e cache local do projeto (`.gradle/`).\n\nConsequência ao apagar: totalmente seguro (equivalente a `./gradlew clean`). Será recriado na próxima compilação.",
            "What it is: build output (`build/`) and project-local Gradle cache (`.gradle/`).\n\nConsequence of deleting: completely safe (equivalent to `./gradlew clean`). Recreated on the next build.",
        ),
        e(
            P,
            "Scala (sbt)",
            "target, project/target (build.sbt)",
            "O que é: pastas `target/` e `project/target/` com classes compiladas pelo sbt.\n\nConsequência ao apagar: totalmente seguro (equivalente a `sbt clean`).",
            "What it is: `target/` and `project/target/` folders containing classes compiled by sbt.\n\nConsequence of deleting: completely safe (equivalent to `sbt clean`).",
        ),
        e(
            P,
            "Clojure",
            "target, .cpcache (project.clj, deps.edn)",
            "O que é: artefatos de compilação (`target/`) e cache de classpath (`.cpcache/`) de projetos Leiningen/Clojure CLI.\n\nConsequência ao apagar: totalmente seguro.",
            "What it is: build artifacts (`target/`) and classpath cache (`.cpcache/`) for Leiningen/Clojure CLI projects.\n\nConsequence of deleting: completely safe.",
        ),
        e(
            P,
            "Python",
            ".venv, venv, __pycache__, .pytest_cache, .mypy_cache, .ruff_cache, .tox, .nox (pyproject.toml, requirements.txt, setup.py, Pipfile)",
            "O que é: ambientes virtuais locais (`.venv`, `venv`, `.tox`, `.nox`), bytecode (`__pycache__`) e caches de ferramentas (`.pytest_cache`, `.mypy_cache`, `.ruff_cache`).\n\nConsequência ao apagar:\n• `__pycache__` e `.*_cache`: totalmente seguro, recriados automaticamente.\n• `.venv` / `venv`: seguro para o código, mas você precisará recriar o virtualenv e reinstalar o `requirements.txt` / `pyproject.toml`.",
            "What it is: local virtual environments (`.venv`, `venv`, `.tox`, `.nox`), bytecode (`__pycache__`) and tool caches (`.pytest_cache`, `.mypy_cache`, `.ruff_cache`).\n\nConsequence of deleting:\n• `__pycache__` and `.*_cache`: completely safe, recreated automatically.\n• `.venv` / `venv`: safe for source code, but you will need to recreate the virtualenv and reinstall dependencies.",
        ),
        e(
            P,
            "Dart / Flutter",
            ".dart_tool, build (pubspec.yaml)",
            "O que é: metadados de pacotes/código gerado (`.dart_tool/`) e saída de compilação (`build/`) de projetos Dart/Flutter.\n\nConsequência ao apagar: totalmente seguro (equivalente a `flutter clean`). Rode `flutter pub get` ao retomar o projeto.",
            "What it is: package metadata/codegen (`.dart_tool/`) and build output (`build/`) for Dart/Flutter projects.\n\nConsequence of deleting: completely safe (equivalent to `flutter clean`). Run `flutter pub get` when returning to the project.",
        ),
        e(
            P,
            "PHP",
            "vendor (composer.json)",
            "O que é: pasta `vendor/` com dependências PHP instaladas pelo Composer.\n\nConsequência ao apagar: seguro para o código. Para rodar o projeto de novo, execute `composer install`.",
            "What it is: `vendor/` directory with PHP dependencies installed by Composer.\n\nConsequence of deleting: safe for source code. Run `composer install` to restore dependencies.",
        ),
        e(
            P,
            "Ruby",
            "vendor/bundle (Gemfile)",
            "O que é: pasta `vendor/bundle` com gems instaladas localmente no projeto pelo Bundler.\n\nConsequência ao apagar: seguro. Execute `bundle install` para restaurar quando precisar.",
            "What it is: `vendor/bundle` directory containing gems installed locally by Bundler.\n\nConsequence of deleting: safe. Run `bundle install` to restore when needed.",
        ),
        e(
            P,
            "Swift / CocoaPods",
            ".build (Package.swift), Pods (Podfile)",
            "O que é: pasta `.build/` do Swift Package Manager e pasta `Pods/` do CocoaPods.\n\nConsequência ao apagar: seguro (a menos que você tenha versionado `Pods` no git, mas itens rastreados pelo git já são ignorados). Rode `swift build` ou `pod install` para recriar.",
            "What it is: `.build/` directory from Swift Package Manager and `Pods/` directory from CocoaPods.\n\nConsequence of deleting: safe (git-tracked folders are automatically skipped). Run `swift build` or `pod install` to recreate.",
        ),
        e(
            P,
            "Elixir / Erlang",
            "_build, deps (mix.exs, rebar.config)",
            "O que é: artefatos compilados BEAM (`_build/`) e dependências baixadas (`deps/`) de projetos Mix/Rebar3.\n\nConsequência ao apagar: seguro. Rode `mix deps.get` e `mix compile` ao voltar ao projeto.",
            "What it is: compiled BEAM artifacts (`_build/`) and fetched dependencies (`deps/`) for Mix/Rebar3 projects.\n\nConsequence of deleting: safe. Run `mix deps.get` and `mix compile` when returning to the project.",
        ),
        e(
            P,
            "Haskell",
            ".stack-work, dist-newstyle (stack.yaml, cabal.project, *.cabal)",
            "O que é: diretórios de saída de compilação do Stack (`.stack-work/`) e do Cabal (`dist-newstyle/`).\n\nConsequência ao apagar: totalmente seguro (equivalente a `stack clean` / `cabal clean`).",
            "What it is: build output directories for Stack (`.stack-work/`) and Cabal (`dist-newstyle/`).\n\nConsequence of deleting: completely safe (equivalent to `stack clean` / `cabal clean`).",
        ),
        e(
            P,
            "Zig",
            "zig-out, .zig-cache (build.zig)",
            "O que é: binários gerados (`zig-out/`) e cache incremental local (`.zig-cache/`) de projetos Zig.\n\nConsequência ao apagar: totalmente seguro. `zig build` recria ambos.",
            "What it is: generated binaries (`zig-out/`) and local incremental cache (`.zig-cache/`) for Zig projects.\n\nConsequence of deleting: completely safe. `zig build` recreates both.",
        ),
        e(
            P,
            "CMake",
            "build, cmake-build-* (CMakeLists.txt)",
            "O que é: diretórios de build out-of-source do CMake (`build/` e `cmake-build-debug`/`release` do CLion).\n\nConsequência ao apagar: seguro. O CMake/IDE reconfigurará e recompilará o projeto do zero.",
            "What it is: out-of-source CMake build directories (`build/` and CLion's `cmake-build-debug`/`release`).\n\nConsequence of deleting: safe. CMake/your IDE will reconfigure and recompile the project from scratch.",
        ),
        e(
            P,
            ".NET",
            "bin, obj (*.csproj, *.fsproj, *.sln)",
            "O que é: pastas `bin/` (binários compilados) e `obj/` (arquivos intermediários/NuGet) de projetos C#/F#/.NET.\n\nConsequência ao apagar: totalmente seguro (equivalente a `dotnet clean`).",
            "What it is: `bin/` (compiled binaries) and `obj/` (intermediate/NuGet files) folders for C#/F#/.NET projects.\n\nConsequence of deleting: completely safe (equivalent to `dotnet clean`).",
        ),
        e(
            P,
            "Terraform",
            ".terraform (*.tf, .terraform.lock.hcl)",
            "O que é: pasta `.terraform/` contendo os binários de providers e módulos baixados por `terraform init` (NÃO é o arquivo `terraform.tfstate`).\n\nConsequência ao apagar: seguro; basta rodar `terraform init` novamente no diretório.",
            "What it is: `.terraform/` directory containing provider binaries and modules downloaded by `terraform init` (NOT your `terraform.tfstate` file).\n\nConsequence of deleting: safe; simply run `terraform init` again in the directory.",
        ),
        e(
            P,
            "Unity",
            "Library, Temp, Obj, Logs (Assets/ + ProjectSettings/)",
            "O que é: cache de assets importados/shaders (`Library/`, geralmente muito grande) e pastas temporárias (`Temp/`, `Obj/`, `Logs/`) de projetos Unity. Seus assets e cenas ficam em `Assets/` e NÃO são tocados.\n\nConsequência ao apagar: seguro, mas abrir o projeto no Unity pela primeira vez depois de apagar `Library/` pode demorar bastante enquanto ele reimporta todos os assets.",
            "What it is: imported asset/shader cache (`Library/`, usually very large) and temporary folders (`Temp/`, `Obj/`, `Logs/`) for Unity projects. Your scenes and assets live in `Assets/` and are NOT touched.\n\nConsequence of deleting: safe, but opening the project in Unity the first time after deleting `Library/` can take a long time while it reimports all assets.",
        ),
        e(
            P,
            "Unreal Engine",
            "Intermediate, Binaries, DerivedDataCache, Saved/Logs (*.uproject)",
            "O que é: arquivos intermediários de compilação (`Intermediate/`), executáveis compilados (`Binaries/`), shaders/assets compilados (`DerivedDataCache/`) e logs (`Saved/Logs/`) da Unreal Engine.\n\nConsequência ao apagar: seguro para o código e `Content/`; na próxima abertura a Unreal recompilará o código C++ e os shaders.",
            "What it is: intermediate build files (`Intermediate/`), compiled binaries (`Binaries/`), compiled shaders/assets (`DerivedDataCache/`) and logs (`Saved/Logs/`) for Unreal Engine.\n\nConsequence of deleting: safe for source code and `Content/`; on next launch Unreal will recompile C++ code and shaders.",
        ),
    ]
}

/// Encontra a entrada de catálogo correspondente a um `Target` específico e devolve uma explicação
/// sobre o que ele é e as consequências de apagá-lo (no idioma atual).
pub fn describe_target(t: &Target) -> String {
    let n = t.name.as_str();
    let entries = catalog();
    let find = |cat: Category, key: &str| -> Option<&'static str> {
        entries
            .iter()
            .find(|e| e.category == cat && e.name.starts_with(key))
            .map(|e| e.info)
    };

    let matched = match t.category {
        Category::Language => {
            if n.starts_with("Cargo ") || n.starts_with("Rustup ") {
                find(Category::Language, "Rust")
            } else if n.starts_with("npm ") {
                find(Category::Language, "npm")
            } else if n.starts_with("pnpm ") {
                find(Category::Language, "pnpm")
            } else if n.starts_with("Yarn ") {
                find(Category::Language, "Yarn")
            } else if n.starts_with("Bun ") {
                find(Category::Language, "Bun")
            } else if n.starts_with("Deno ") {
                find(Category::Language, "Deno")
            } else if n.starts_with("node-gyp") || n.starts_with("nvm ") {
                find(Category::Language, "node-gyp")
            } else if n.starts_with("pip ") || n.starts_with("pdm ") || n.starts_with("pipenv ") {
                find(Category::Language, "pip")
            } else if n.starts_with("Poetry ") {
                find(Category::Language, "Poetry")
            } else if n.starts_with("uv ") {
                find(Category::Language, "uv")
            } else if n.starts_with("Conda ") {
                find(Category::Language, "Conda")
            } else if n.starts_with("Maven ") {
                find(Category::Language, "Maven")
            } else if n.starts_with("Gradle ") {
                find(Category::Language, "Gradle")
            } else if n.starts_with("sbt ") || n.starts_with("Coursier ") {
                find(Category::Language, "Scala")
            } else if n.starts_with("Kotlin ") {
                find(Category::Language, "Kotlin")
            } else if n.starts_with("Go ") {
                find(Category::Language, "Go")
            } else if n.starts_with("NuGet ") {
                find(Category::Language, ".NET")
            } else if n.starts_with("Dart/") {
                find(Category::Language, "Dart")
            } else if n.starts_with("Composer ") {
                find(Category::Language, "PHP")
            } else if n.starts_with("RubyGems") || n.starts_with("Bundler ") {
                find(Category::Language, "Ruby")
            } else if n.starts_with("Hex ") || n.starts_with("Mix ") || n.starts_with("Rebar3 ") {
                find(Category::Language, "Elixir")
            } else if n.starts_with("Cabal ") || n.starts_with("Stack ") || n.starts_with("GHCup ")
            {
                find(Category::Language, "Haskell")
            } else if n.starts_with("Zig ") {
                find(Category::Language, "Zig")
            } else if n.starts_with("Julia ") {
                find(Category::Language, "Julia")
            } else if n.starts_with("Nimble ") {
                find(Category::Language, "Nim")
            } else if n.starts_with("Conan") || n.starts_with("vcpkg ") {
                find(Category::Language, "C / C++")
            } else if n.starts_with("LuaRocks ") {
                find(Category::Language, "Lua")
            } else if n.starts_with("CPANM ") {
                find(Category::Language, "Perl")
            } else {
                None
            }
        }
        Category::Tool => {
            if n.starts_with("Homebrew ") {
                find(Category::Tool, "Homebrew")
            } else if n.starts_with("Xcode ") {
                find(Category::Tool, "Xcode")
            } else if n.starts_with("CocoaPods ") || n.starts_with("Swift Package") {
                find(Category::Tool, "CocoaPods")
            } else if n.starts_with("Android SDK") || n.starts_with("Android build-cache") {
                find(Category::Tool, "Android SDK")
            } else if n.starts_with("Android AVD") {
                find(Category::Tool, "Android emulators")
            } else if n.starts_with("Expo ") {
                find(Category::Tool, "Expo")
            } else if n.starts_with("Docker buildx") {
                find(Category::Tool, "Docker buildx")
            } else if n.starts_with("Docker images") {
                find(Category::Tool, "Docker images")
            } else if n.starts_with("Docker volumes") {
                find(Category::Tool, "Docker volumes")
            } else if n.starts_with("Playwright ")
                || n.starts_with("Puppeteer ")
                || n.starts_with("Cypress ")
            {
                find(Category::Tool, "Playwright")
            } else if n.starts_with("Electron") {
                find(Category::Tool, "Electron")
            } else if n == "ccache" || n == "sccache" || n.starts_with("Bazel ") {
                find(Category::Tool, "ccache")
            } else if n.starts_with("pre-commit ")
                || n.starts_with("Turborepo ")
                || n.starts_with("Nx ")
            {
                find(Category::Tool, "pre-commit")
            } else if n.starts_with("Terraform ")
                || n.starts_with("Kubernetes ")
                || n.starts_with("Helm ")
                || n.starts_with("AWS ")
            {
                find(Category::Tool, "Terraform")
            } else if n.starts_with("Vagrant ") || n.starts_with("Minikube ") {
                find(Category::Tool, "Vagrant")
            } else if n.starts_with("Hugging Face")
                || n.starts_with("Ollama ")
                || n.starts_with("PyTorch ")
                || n.starts_with("Keras ")
                || n.starts_with("LM Studio")
            {
                find(Category::Tool, "AI / ML")
            } else {
                None
            }
        }
        Category::Ide => {
            if n.starts_with("VS Code")
                || n.starts_with("Code/")
                || n.starts_with("Code - Insiders/")
            {
                find(Category::Ide, "VS Code")
            } else if n.starts_with("Cursor")
                || n.starts_with("Windsurf")
                || n.starts_with("Trae")
                || n.starts_with("VSCodium")
            {
                find(Category::Ide, "Cursor")
            } else if n.starts_with("JetBrains") {
                find(Category::Ide, "JetBrains")
            } else if n.starts_with("Android Studio") {
                find(Category::Ide, "Android Studio")
            } else if n.starts_with("Visual Studio") {
                find(Category::Ide, "Visual Studio")
            } else if n.starts_with("Eclipse") {
                find(Category::Ide, "Eclipse")
            } else if n.starts_with("Neovim") {
                find(Category::Ide, "Neovim")
            } else if n.starts_with("Sublime Text") {
                find(Category::Ide, "Sublime Text")
            } else if n.starts_with("Zed") {
                find(Category::Ide, "Zed")
            } else {
                None
            }
        }
        Category::LocalProject => {
            let prefix = n.split(':').next().unwrap_or("");
            match prefix {
                "Rust" => find(Category::LocalProject, "Rust"),
                "Node" | "Next.js" | "Nuxt" | "Angular" | "SvelteKit" | "Astro" | "Nitro/Nuxt"
                | "Turborepo" | "Parcel" => find(Category::LocalProject, "Node / Web"),
                "Maven" => find(Category::LocalProject, "Maven"),
                "Gradle" => find(Category::LocalProject, "Gradle"),
                "Scala (sbt)" => find(Category::LocalProject, "Scala"),
                "Clojure" => find(Category::LocalProject, "Clojure"),
                "Python" => find(Category::LocalProject, "Python"),
                "Dart" => find(Category::LocalProject, "Dart"),
                "PHP" => find(Category::LocalProject, "PHP"),
                "Ruby" => find(Category::LocalProject, "Ruby"),
                "Swift" | "CocoaPods" => find(Category::LocalProject, "Swift"),
                "Elixir" | "Erlang" => find(Category::LocalProject, "Elixir"),
                "Haskell" => find(Category::LocalProject, "Haskell"),
                "Zig" => find(Category::LocalProject, "Zig"),
                "CMake" => find(Category::LocalProject, "CMake"),
                ".NET" => find(Category::LocalProject, ".NET"),
                "Terraform" => find(Category::LocalProject, "Terraform"),
                "Unity" => find(Category::LocalProject, "Unity"),
                "Unreal" => find(Category::LocalProject, "Unreal"),
                _ => None,
            }
        }
    };

    if let Some(text) = matched {
        return text.to_string();
    }

    // Fallback para alvos extras definidos pelo usuário no `config.toml`
    if lang() == Lang::Pt {
        format!(
            "Alvo configurado manualmente ({}).\n\nCaminho: {}\n\nConsequência ao apagar: o diretório acima será removido{}.",
            t.category.label(),
            t.path.display(),
            if t.critical {
                " (marcado como item crítico)"
            } else {
                ""
            }
        )
    } else {
        format!(
            "Custom target ({}).\n\nPath: {}\n\nConsequence of deleting: the directory above will be removed{}.",
            t.category.label(),
            t.path.display(),
            if t.critical {
                " (marked as a critical item)"
            } else {
                ""
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn catalog_covers_every_category() {
        let c = catalog();
        for cat in [
            Category::Language,
            Category::Tool,
            Category::Ide,
            Category::LocalProject,
        ] {
            assert!(c.iter().any(|x| x.category == cat && !x.info.is_empty()));
        }
    }

    #[test]
    fn describes_builtin_and_custom_targets() {
        let t = Target::new(
            Category::Language,
            "Cargo registry",
            PathBuf::from("/x"),
            false,
        );
        assert!(describe_target(&t).contains("Cargo"));
        let custom = Target::new(
            Category::Tool,
            "Meu cache customizado",
            PathBuf::from("/meu/cache"),
            true,
        );
        assert!(describe_target(&custom).contains("/meu/cache"));
    }
}
