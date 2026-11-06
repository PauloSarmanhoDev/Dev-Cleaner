mod ui;

use anyhow::Result;
use clap::Parser;
use dev_cleaner_core::{cleaner, discovery, model::format_size, scanner};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Parser)]
#[command(version, about = "Limpa caches e artefatos de linguagens, IDEs e projetos locais")]
struct Cli {
    /// Diretório raiz para procurar projetos locais
    #[arg(short, long, default_value = ".")]
    root: PathBuf,
    /// Profundidade máxima da busca por projetos
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let warnings = Mutex::new(Vec::<String>::new());

    let mut targets = Vec::new();
    if !cli.no_lang {
        targets.extend(discovery::language_targets());
    }
    if !cli.no_ide {
        targets.extend(discovery::ide_targets());
    }
    if !cli.no_local {
        targets.extend(discovery::local_targets(&cli.root, cli.depth));
    }

    println!("Analisando {} diretórios candidatos...", targets.len());
    let pb = ProgressBar::new(targets.len() as u64);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let targets = scanner::scan(targets, &warnings, |p| {
        pb.set_position(p.done as u64);
        pb.set_message(format!("{} encontrados", format_size(p.bytes)));
    });
    pb.finish_and_clear();

    if targets.is_empty() {
        println!("Nada para limpar.");
    } else {
        ui::print_table(&targets);
        if !cli.dry_run {
            let idx = ui::select(&targets)?;
            let chosen: Vec<_> = idx.iter().map(|&i| &targets[i]).collect();
            if chosen.is_empty() {
                println!("Nada selecionado.");
            } else if ui::confirm(&chosen)? {
                let owned: Vec<_> = chosen.into_iter().cloned().collect();
                let freed = cleaner::clean(&owned, &warnings, |_, _| {});
                println!("Liberado: {}", format_size(freed));
            }
        }
    }

    let w = warnings.into_inner().unwrap();
    if !w.is_empty() {
        eprintln!("\n{} aviso(s) (arquivos bloqueados/sem permissão foram ignorados):", w.len());
        for m in w.iter().take(20) {
            eprintln!("  ! {m}");
        }
        if w.len() > 20 {
            eprintln!("  ... e mais {}", w.len() - 20);
        }
    }
    Ok(())
}

