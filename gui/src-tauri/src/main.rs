// Evita abrir um console extra no Windows em release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use scour_core::{
    CancelToken, Warnings, catalog,
    cleaner::{self, CleanOptions, DeleteMode},
    config, i18n,
    i18n::{Lang, t, tf},
    model::{Target, parse_age},
    pipeline::{self, Selection},
    safety,
    scanner::{self, ScanProgress},
    store,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State, async_runtime::spawn_blocking};

/// Último resultado de varredura. A UI só referencia itens por índice,
/// então o frontend nunca consegue pedir a remoção de um caminho arbitrário.
#[derive(Default)]
struct AppState {
    targets: Mutex<Vec<Target>>,
    cancel: CancelToken,
}

#[derive(Deserialize)]
struct ScanOptions {
    lang: bool,
    ide: bool,
    /// Se vazio, projetos locais não são procurados.
    root: Option<String>,
    depth: usize,
    /// Ex.: "30d". Vazio = sem filtro.
    older_than: Option<String>,
}

#[derive(Serialize)]
struct ScanResult {
    targets: Vec<Target>,
    warnings: Vec<String>,
    cancelled: bool,
    /// Idade do resultado em minutos, quando veio do cache.
    cached_minutes: Option<u64>,
}

#[derive(Serialize)]
struct CleanResult {
    freed: u64,
    removed: usize,
    skipped: usize,
    warnings: Vec<String>,
    trashed: bool,
}

#[derive(Clone, Serialize)]
struct CleanProgress {
    done: usize,
    total: usize,
}

#[derive(Serialize)]
struct Info {
    lang: &'static str,
    audit_log: Option<String>,
    config_path: Option<String>,
    permanent_default: bool,
    older_than_default: Option<String>,
}

#[derive(Serialize)]
struct ConfigView {
    config: config::Config,
    path: Option<String>,
    /// Aviso de leitura (ex.: TOML inválido, caso em que o padrão é mostrado).
    warning: Option<String>,
}

fn take_warnings(w: Warnings) -> Vec<String> {
    w.into_inner().unwrap_or_default()
}

#[tauri::command]
fn get_info() -> Info {
    let (cfg, _) = config::load(None);
    Info {
        lang: i18n::lang().code(),
        audit_log: store::audit_path().map(|p| p.display().to_string()),
        config_path: config::default_path().map(|p| p.display().to_string()),
        permanent_default: cfg.permanent,
        older_than_default: cfg.older_than,
    }
}

/// Lista do que o programa sabe detectar.
#[tauri::command]
fn get_supported() -> Vec<catalog::CatalogEntry> {
    catalog::catalog()
}

/// Nomes e descrições dos alvos do último resultado, no idioma atual (mesma ordem/índices).
#[tauri::command]
fn target_names(state: State<'_, AppState>) -> Vec<String> {
    state
        .targets
        .lock()
        .unwrap()
        .iter()
        .map(Target::display_name)
        .collect()
}

/// Explicação sobre o que cada alvo do último resultado faz e a consequência de apagá-lo.
#[tauri::command]
fn target_infos(state: State<'_, AppState>) -> Vec<String> {
    state
        .targets
        .lock()
        .unwrap()
        .iter()
        .map(catalog::describe_target)
        .collect()
}

#[tauri::command]
fn get_config() -> ConfigView {
    let (config, warning) = config::load(None);
    ConfigView {
        config,
        path: config::default_path().map(|p| p.display().to_string()),
        warning,
    }
}

#[tauri::command]
fn save_config(mut config: config::Config) -> Result<String, String> {
    config.validate()?;
    config::save(None, &config).map(|p| p.display().to_string())
}

#[tauri::command]
fn set_lang(lang: String) {
    if let Some(l) = Lang::parse(&lang) {
        i18n::set_lang(l);
    }
}

#[tauri::command]
fn cancel(state: State<'_, AppState>) {
    state.cancel.cancel();
}

/// Carrega o último resultado salvo (sem varrer de novo).
#[tauri::command]
async fn load_cache(state: State<'_, AppState>) -> Result<ScanResult, String> {
    let (mins, mut list) = store::load().ok_or_else(|| t("gui.cache_none").to_string())?;
    spawn_blocking(move || {
        scour_core::procs::annotate(&mut list);
        list
    })
    .await
    .map(|list| {
        *state.targets.lock().unwrap() = list.clone();
        ScanResult {
            targets: list,
            warnings: vec![],
            cancelled: false,
            cached_minutes: Some(mins),
        }
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan(
    app: AppHandle,
    state: State<'_, AppState>,
    opts: ScanOptions,
) -> Result<ScanResult, String> {
    state.cancel.reset();
    let cancel = state.cancel.clone();
    let age = match opts
        .older_than
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) => Some(parse_age(s).ok_or_else(|| format!("older_than: '{s}'"))?),
        None => None,
    };

    let (targets, warnings, cancelled) = spawn_blocking(move || {
        let warnings = Warnings::default();
        let (cfg, cfg_err) = config::load(None);
        if let Some(e) = cfg_err {
            warnings.lock().unwrap().push(e);
        }
        let sel = Selection {
            lang: opts.lang,
            ide: opts.ide,
            root: opts
                .root
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .map(PathBuf::from),
            depth: opts.depth,
        };
        let candidates = pipeline::candidates(&sel, &cfg, &warnings);
        let list = scanner::scan(candidates, &warnings, &cancel, |p: ScanProgress| {
            let _ = app.emit("scan-progress", p);
        });
        let cancelled = cancel.is_cancelled();
        if !cancelled {
            store::save(&list);
        }
        let list = match age {
            Some(secs) => pipeline::filter_older_than(list, secs),
            None => list,
        };
        (list, take_warnings(warnings), cancelled)
    })
    .await
    .map_err(|e| e.to_string())?;

    *state.targets.lock().unwrap() = targets.clone();
    Ok(ScanResult {
        targets,
        warnings,
        cancelled,
        cached_minutes: None,
    })
}

#[tauri::command]
async fn clean(
    app: AppHandle,
    state: State<'_, AppState>,
    indices: Vec<usize>,
    confirmation: Option<String>,
    permanent: bool,
    force_running: bool,
) -> Result<CleanResult, String> {
    let chosen: Vec<Target> = {
        let all = state.targets.lock().unwrap();
        let mut idx = indices;
        idx.sort_unstable();
        idx.dedup();
        idx.into_iter()
            .map(|i| all.get(i).cloned().ok_or_else(|| tf("err.index", &[&i])))
            .collect::<Result<_, _>>()?
    };
    if chosen.is_empty() {
        return Err(t("err.nothing").into());
    }
    // As verificações também são feitas aqui: não dependem só da UI.
    safety::check_critical(&chosen.iter().collect::<Vec<_>>(), confirmation.as_deref())?;
    if !force_running && let Some(r) = chosen.iter().find_map(|x| x.running.as_ref()) {
        return Err(tf("err.running", &[r]));
    }

    state.cancel.reset();
    let cancel = state.cancel.clone();
    let mode = if permanent {
        DeleteMode::Permanent
    } else {
        DeleteMode::Trash
    };
    spawn_blocking(move || {
        let warnings = Warnings::default();
        let opts = CleanOptions {
            mode,
            threads: 4,
            cancel,
        };
        let rep = cleaner::clean(&chosen, &warnings, &opts, |done, total| {
            let _ = app.emit("clean-progress", CleanProgress { done, total });
        });
        CleanResult {
            freed: rep.freed,
            removed: rep.removed,
            skipped: rep.skipped,
            warnings: take_warnings(warnings),
            trashed: mode == DeleteMode::Trash,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

fn main() {
    let (cfg, _) = config::load(None);
    i18n::set_lang(
        cfg.language
            .as_deref()
            .and_then(Lang::parse)
            .unwrap_or_else(i18n::detect),
    );
    store::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_info,
            get_supported,
            target_names,
            target_infos,
            get_config,
            save_config,
            set_lang,
            cancel,
            load_cache,
            scan,
            clean
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Scour");
}
