//! Tabela e menus interativos.

use dev_cleaner_core::model::{format_size, Target};
use anyhow::Result;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect};

pub fn print_table(targets: &[Target]) {
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(["#", "Tipo", "Nome", "Caminho", "Arquivos", "Tamanho"]);
    for (i, t) in targets.iter().enumerate() {
        let mut name = Cell::new(&t.name);
        if t.critical {
            name = name.fg(Color::Red);
        }
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(t.category.label()),
            name,
            Cell::new(t.path.display()),
            Cell::new(t.files),
            Cell::new(format_size(t.size)),
        ]);
    }
    println!("{table}");
    let total: u64 = targets.iter().map(|t| t.size).sum();
    println!("Total recuperável: {}   (vermelho = crítico, exige confirmação)", format_size(total));
}

/// Retorna os índices escolhidos.
pub fn select(targets: &[Target]) -> Result<Vec<usize>> {
    let items: Vec<String> = targets
        .iter()
        .map(|t| {
            format!(
                "[{}] {} — {} ({}){}",
                t.category.label(),
                t.name,
                format_size(t.size),
                t.path.display(),
                if t.critical { "  ⚠ CRÍTICO" } else { "" }
            )
        })
        .collect();
    Ok(MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Selecione o que remover (espaço marca, enter confirma)")
        .items(&items)
        .interact()?)
}

/// Confirmação geral + confirmação reforçada para itens críticos.
pub fn confirm(chosen: &[&Target]) -> Result<bool> {
    let total: u64 = chosen.iter().map(|t| t.size).sum();
    let theme = ColorfulTheme::default();
    if !Confirm::with_theme(&theme)
        .with_prompt(format!("Remover {} item(ns), liberando {}?", chosen.len(), format_size(total)))
        .default(false)
        .interact()?
    {
        return Ok(false);
    }
    let critical: Vec<_> = chosen.iter().filter(|t| t.critical).collect();
    if !critical.is_empty() {
        println!("\n⚠ Itens críticos selecionados (extensões de IDE serão perdidas e precisarão ser reinstaladas):");
        for t in &critical {
            println!("   - {} ({})", t.name, t.path.display());
        }
        let ans: String = Input::with_theme(&theme)
            .with_prompt("Digite APAGAR para confirmar")
            .allow_empty(true)
            .interact_text()?;
        if ans.trim() != "APAGAR" {
            println!("Confirmação não recebida; operação cancelada.");
            return Ok(false);
        }
    }
    Ok(true)
}

