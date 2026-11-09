//! Tabela e menus interativos.

use anyhow::Result;
use comfy_table::{Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use dialoguer::{Confirm, Input, MultiSelect, theme::ColorfulTheme};
use scour_core::cleaner::DeleteMode;
use scour_core::i18n::{t, tf};
use scour_core::model::{Target, format_size};
use scour_core::safety::CRITICAL_PHRASES;

/// Tabela do que o programa sabe detectar.
pub fn print_catalog(entries: &[scour_core::catalog::CatalogEntry]) {
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header([t("ui.h_type"), t("ui.h_name"), t("ui.h_what")]);
    for e in entries {
        table.add_row(vec![e.category.label(), e.name, e.detail]);
    }
    println!("{}", tf("cli.supported_title", &[&entries.len()]));
    println!("{table}");
}

pub fn print_table(targets: &[Target]) {
    let mut table = Table::new();
    table
        .load_style(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header([
            "#",
            t("ui.h_type"),
            t("ui.h_name"),
            t("ui.h_path"),
            t("ui.h_files"),
            t("ui.h_size"),
            t("ui.h_used"),
        ]);
    for (i, tg) in targets.iter().enumerate() {
        let label = if tg.running.is_some() {
            format!("🔒 {}", tg.display_name())
        } else {
            tg.display_name()
        };
        let mut name = Cell::new(label);
        if tg.critical {
            name = name.fg(Color::Red);
        }
        let used = tg
            .age_days()
            .map(|d| tf("ui.days_ago", &[&d]))
            .unwrap_or_else(|| "-".into());
        table.add_row(vec![
            Cell::new(i + 1),
            Cell::new(tg.category.label()),
            name,
            Cell::new(tg.path.display()),
            Cell::new(tg.files),
            Cell::new(format_size(tg.size)),
            Cell::new(used),
        ]);
    }
    println!("{table}");
    let total: u64 = targets.iter().map(|x| x.size).sum();
    println!("{}", tf("ui.total", &[&format_size(total)]));
}

/// Retorna os índices escolhidos.
pub fn select(targets: &[Target]) -> Result<Vec<usize>> {
    let items: Vec<String> = targets
        .iter()
        .map(|x| {
            format!(
                "[{}] {} — {} ({}){}{}",
                x.category.label(),
                x.display_name(),
                format_size(x.size),
                x.path.display(),
                if x.critical {
                    t("ui.critical_tag").to_string()
                } else {
                    String::new()
                },
                x.running
                    .as_ref()
                    .map(|r| tf("ui.running_tag", &[r]))
                    .unwrap_or_default(),
            )
        })
        .collect();
    Ok(MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(t("ui.select_prompt"))
        .items(&items)
        .interact()?)
}

/// Confirmação geral + aviso de IDE aberta + confirmação reforçada para itens críticos.
pub fn confirm(chosen: &[&Target], mode: DeleteMode) -> Result<bool> {
    let total: u64 = chosen.iter().map(|x| x.size).sum();
    let theme = ColorfulTheme::default();
    let mode_txt = if mode == DeleteMode::Trash {
        t("ui.mode_trash")
    } else {
        t("ui.mode_perm")
    };
    if !Confirm::with_theme(&theme)
        .with_prompt(tf(
            "ui.confirm",
            &[&chosen.len(), &format_size(total), &mode_txt],
        ))
        .default(false)
        .interact()?
    {
        return Ok(false);
    }

    let running: Vec<_> = chosen.iter().filter(|x| x.running.is_some()).collect();
    if !running.is_empty() {
        println!("\n{}", t("ui.running_warn"));
        for x in &running {
            println!(
                "   - {} ({})",
                x.running.as_deref().unwrap_or(""),
                x.display_name()
            );
        }
        if !Confirm::with_theme(&theme)
            .with_prompt(t("ui.running_continue"))
            .default(false)
            .interact()?
        {
            return Ok(false);
        }
    }

    let critical: Vec<_> = chosen.iter().filter(|x| x.critical).collect();
    if !critical.is_empty() {
        println!("\n{}", t("ui.critical_warn"));
        for x in &critical {
            println!("   - {} ({})", x.display_name(), x.path.display());
        }
        let ans: String = Input::with_theme(&theme)
            .with_prompt(tf("ui.critical_prompt", &[&t("critical.word")]))
            .allow_empty(true)
            .interact_text()?;
        if !CRITICAL_PHRASES.contains(&ans.trim()) {
            println!("{}", t("ui.critical_abort"));
            return Ok(false);
        }
    }
    Ok(true)
}
