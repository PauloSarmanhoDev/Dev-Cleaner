mod ui;

use anyhow::{Result, bail};
use clap::Parser;
use dev_cleaner_core::cleaner::{self, CleanOptions, DeleteMode};
use dev_cleaner_core::i18n::{self, Lang, t, tf};
use dev_cleaner_core::model::{Target, format_size, parse_age};
use dev_cleaner_core::pipeline::{self, Selection};
use dev_cleaner_core::{CancelToken, Warnings, config, scanner, store};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::IsTerminal;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Cleans caches and artifacts of languages, IDEs and local projects / Limpa caches e artefatos"
)]
struct Cli {
    /// Diretório raiz para procurar projetos locais / Root folder to look for local projects
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
    /// Profundidade máxima da busca por projetos / Max project search depth
    #[arg(short, long, default_value_t = 6)]
    depth: usize,
    /// Não procurar projetos locais
    #[arg(long)]
    no_local: bool,
    /// Não incluir IDEs/editores
    #[arg(long)]
    no_ide: bool,
    /// Não incluir caches de linguagens
    #[arg(long)]
    no_lang: bool,
    /// Apenas listar, sem remover
    #[arg(long)]
    dry_run: bool,
    /// Só itens sem uso há pelo menos este tempo (ex.: 30d, 2w, 12h)
    #[arg(long, value_name = "AGE")]
    older_than: Option<String>,
    /// Apagar definitivamente em vez de enviar para a lixeira
    #[arg(long)]
    permanent: bool,
    /// Remoções em paralelo (limita a carga no disco)
    #[arg(short, long, default_value_t = 4)]
    jobs: usize,
    /// Não perguntar; exige --select ou --all
    #[arg(short, long)]
    yes: bool,
    /// Com --yes: seleciona itens cujo nome/caminho contém o texto (repetível)
    #[arg(long, value_name = "TEXT")]
    select: Vec<String>,
    /// Com --yes: seleciona todos os itens listados
    #[arg(long)]
    all: bool,
    /// Com --yes: permite remover itens críticos (extensões de IDE)
    #[arg(long)]
    include_critical: bool,
    /// Com --yes: remove mesmo se a IDE estiver aberta
    #[arg(long)]
    force_running: bool,
    /// Imprime a lista em JSON
    #[arg(long)]
    json: bool,
    /// Reutiliza o último resultado salvo em vez de varrer de novo
    #[arg(long)]
    cached: bool,
    /// Arquivo de configuração (padrão: <config>/dev-cleaner/config.toml)
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
    /// Idioma: pt ou en
    #[arg(long, value_name = "LANG")]
    lang: Option<String>,
}

/// Índices dos alvos cujo nome ou caminho contém algum dos textos (sem diferenciar maiúsculas).
fn matching(targets: &[Target], patterns: &[String]) -> Vec<usize> {
    let pats: Vec<String> = patterns.iter().map(|p| p.to_lowercase()).collect();
    targets
        .iter()
        .enumerate()
        .filter(|(_, x)| {
            let hay = format!("{} {}", x.name, x.path.display()).to_lowercase();
            pats.iter().any(|p| hay.contains(p))
        })
        .map(|(i, _)| i)
        .collect()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let warnings = Warnings::default();

    let (cfg, cfg_err) = config::load(cli.config.as_deref());
    if let Some(e) = cfg_err {
        warnings.lock().unwrap().push(e);
    }
    let lang = cli
        .lang
        .as_deref()
        .and_then(Lang::parse)
        .or_else(|| cfg.language.as_deref().and_then(Lang::parse))
        .unwrap_or_else(i18n::detect);
    i18n::set_lang(lang);
    let audit = store::init_logging();

    let age = match cli.older_than.as_deref().or(cfg.older_than.as_deref()) {
        Some(s) => Some(parse_age(s).ok_or_else(|| anyhow::anyhow!("--older-than: '{s}'"))?),
        None => None,
    };
    let mode = if cli.permanent || cfg.permanent {
        DeleteMode::Permanent
    } else {
        DeleteMode::Trash
    };

    let cancel = CancelToken::default();
    {
        let c = cancel.clone();
        let _ = ctrlc::set_handler(move || {
            if c.is_cancelled() {
                std::process::exit(130);
            }
            c.cancel();
        });
    }

    // 1. Obter a lista (varredura ou cache)
    let mut targets = None;
    if cli.cached {
        match store::load() {
            Some((mins, mut list)) => {
                println!("{}", tf("cli.cached", &[&mins]));
                dev_cleaner_core::procs::annotate(&mut list);
                targets = Some(list);
            }
            None => println!("{}", t("cli.no_cache")),
        }
    }
    let targets = match targets {
        Some(l) => l,
        None => {
            let sel = Selection {
                lang: !cli.no_lang,
                ide: !cli.no_ide,
                root: (!cli.no_local).then(|| cli.root.clone()),
                depth: cli.depth,
            };
            let candidates = pipeline::candidates(&sel, &cfg, &warnings);
            println!("{}", tf("cli.analyzing", &[&candidates.len()]));
            let pb = ProgressBar::new(candidates.len() as u64);
            pb.set_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:30.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("=> "),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let list = scanner::scan(candidates, &warnings, &cancel, |p| {
                pb.set_position(p.done as u64);
                pb.set_message(format!(
                    "{} • {} files",
                    tf("cli.found", &[&format_size(p.bytes)]),
                    p.files
                ));
            });
            pb.finish_and_clear();
            if cancel.is_cancelled() {
                println!("{}", t("cli.cancelled"));
                return Ok(());
            }
            store::save(&list);
            list
        }
    };
    let targets = match age {
        Some(secs) => pipeline::filter_older_than(targets, secs),
        None => targets,
    };

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&targets)?);
    }

    if targets.is_empty() {
        println!("{}", t("cli.nothing_clean"));
    } else if cli.dry_run || (cli.json && !cli.yes) {
        if !cli.json {
            ui::print_table(&targets);
        }
    } else {
        if !cli.json {
            ui::print_table(&targets);
        }
        // 2. Selecionar
        let chosen: Vec<&Target> = if cli.yes {
            if !cli.all && cli.select.is_empty() {
                bail!(t("cli.need_select"));
            }
            let idx = if cli.all {
                (0..targets.len()).collect()
            } else {
                matching(&targets, &cli.select)
            };
            idx.into_iter()
                .map(|i| &targets[i])
                .filter(|x| {
                    if x.critical && !cli.include_critical {
                        eprintln!("{}", tf("cli.skip_critical", &[&x.name]));
                        return false;
                    }
                    if let (Some(app), false) = (&x.running, cli.force_running) {
                        eprintln!("{}", tf("cli.skip_running", &[app, &x.name]));
                        return false;
                    }
                    true
                })
                .collect()
        } else {
            if !std::io::stdin().is_terminal() {
                bail!(t("cli.no_tty"));
            }
            let idx = ui::select(&targets)?;
            let c: Vec<&Target> = idx.iter().map(|&i| &targets[i]).collect();
            if !c.is_empty() && !ui::confirm(&c, mode)? {
                println!("{}", t("cli.cancelled"));
                Vec::new()
            } else {
                c
            }
        };

        // 3. Remover
        if chosen.is_empty() {
            println!("{}", t("cli.none_selected"));
        } else {
            cancel.reset();
            let owned: Vec<Target> = chosen.into_iter().cloned().collect();
            let pb = ProgressBar::new(owned.len() as u64);
            pb.set_style(
                ProgressStyle::with_template("{spinner:.red} [{bar:30.red/yellow}] {pos}/{len}")
                    .unwrap(),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            let opts = CleanOptions {
                mode,
                threads: cli.jobs,
                cancel: cancel.clone(),
            };
            let rep = cleaner::clean(&owned, &warnings, &opts, |d, _| pb.set_position(d as u64));
            pb.finish_and_clear();
            println!(
                "{}",
                tf("cli.freed", &[&format_size(rep.freed), &rep.removed])
            );
            if mode == DeleteMode::Trash && rep.removed > 0 {
                println!("{}", t("cli.to_trash"));
            }
        }
    }

    let w = warnings.into_inner().unwrap();
    if !w.is_empty() {
        eprintln!("\n{}", tf("cli.warnings", &[&w.len()]));
        for m in w.iter().take(20) {
            eprintln!("  ! {m}");
        }
        if w.len() > 20 {
            eprintln!("  {}", tf("cli.more", &[&(w.len() - 20)]));
        }
    }
    if let Some(a) = audit {
        eprintln!("{}", tf("cli.audit", &[&a.display()]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dev_cleaner_core::model::Category;

    #[test]
    fn select_matches_name_or_path_case_insensitively() {
        let mk = |n: &str, p: &str| Target::new(Category::Tool, n, PathBuf::from(p), false);
        let ts = vec![
            mk("npm cache", "/a/npm"),
            mk("Cargo registry", "/b/cargo"),
            mk("x", "/c/NODE_MODULES"),
        ];
        assert_eq!(matching(&ts, &["NPM".into()]), vec![0]);
        assert_eq!(
            matching(&ts, &["node_modules".into(), "cargo".into()]),
            vec![1, 2]
        );
        assert!(matching(&ts, &["zzz".into()]).is_empty());
    }
}
