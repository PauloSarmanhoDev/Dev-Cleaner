// Evita abrir um console extra no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dev_cleaner_core::{
    cleaner, discovery,
    model::Target,
    scanner::{self, ScanProgress},
    Warnings,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{async_runtime::spawn_blocking, AppHandle, Emitter, State};

/// Palavra que o usuário precisa digitar para apagar itens críticos.
const CRITICAL_PHRASE: &str = "APAGAR";

/// Último resultado de varredura. A UI só referencia itens por índice,
/// então o frontend nunca consegue pedir a remoção de um caminho arbitrário.
#[derive(Default)]
struct AppState {
    targets: Mutex<Vec<Target>>,
}

#[derive(Deserialize)]
struct ScanOptions {
    lang: bool,
    ide: bool,
    /// Se vazio, projetos locais não são procurados.
    root: Option<String>,
    depth: usize,
}

#[derive(Serialize)]
struct ScanResult {
    targets: Vec<Target>,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct CleanResult {
    freed: u64,
    warnings: Vec<String>,
}

#[derive(Clone, Serialize)]
struct CleanProgress {
    done: usize,
    total: usize,
}

fn take_warnings(w: Warnings) -> Vec<String> {
    w.into_inner().unwrap_or_default()
}

#[tauri::command]
async fn scan(
    app: AppHandle,
    state: State<'_, AppState>,
    opts: ScanOptions,
) -> Result<ScanResult, String> {
    let (targets, warnings) = spawn_blocking(move || {
        let warnings = Warnings::default();
        let mut candidates = Vec::new();
        if opts.lang {
            candidates.extend(discovery::language_targets());
        }
        if opts.ide {
            candidates.extend(discovery::ide_targets());
        }
        if let Some(root) = opts.root.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
            let root = PathBuf::from(root);
            if root.is_dir() {
                candidates.extend(discovery::local_targets(&root, opts.depth.clamp(1, 20)));
            } else {
                warnings.lock().unwrap().push(format!("Pasta raiz inválida: {}", root.display()));
            }
        }
        let targets = scanner::scan(candidates, &warnings, |p: ScanProgress| {
            let _ = app.emit("scan-progress", p);
        });
        (targets, take_warnings(warnings))
    })
    .await
    .map_err(|e| e.to_string())?;

    *state.targets.lock().unwrap() = targets.clone();
    Ok(ScanResult { targets, warnings })
}

#[tauri::command]
async fn clean(
    app: AppHandle,
    state: State<'_, AppState>,
    indices: Vec<usize>,
    confirmation: Option<String>,
) -> Result<CleanResult, String> {
    let chosen: Vec<Target> = {
        let all = state.targets.lock().unwrap();
        let mut idx = indices;
        idx.sort_unstable();
        idx.dedup();
        idx.into_iter()
            .map(|i| all.get(i).cloned().ok_or_else(|| format!("Índice inválido: {i}")))
            .collect::<Result<_, _>>()?
    };
    if chosen.is_empty() {
        return Err("Nada selecionado.".into());
    }
    // A verificação também é feita aqui: não depende só da UI.
    if chosen.iter().any(|t| t.critical)
        && confirmation.as_deref().map(str::trim) != Some(CRITICAL_PHRASE)
    {
        return Err(format!("Itens críticos exigem digitar {CRITICAL_PHRASE}."));
    }

    spawn_blocking(move || {
        let warnings = Warnings::default();
        let freed = cleaner::clean(&chosen, &warnings, |done, total| {
            let _ = app.emit("clean-progress", CleanProgress { done, total });
        });
        CleanResult { freed, warnings: take_warnings(warnings) }
    })
    .await
    .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![scan, clean])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Dev Cleaner");
}

