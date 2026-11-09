//! Cache do último resultado de varredura e registro de auditoria.

use crate::model::{Target, now_secs};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Serialize, Deserialize)]
struct Saved {
    saved_at: u64,
    targets: Vec<Target>,
}

fn base_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("scour"))
}

pub fn save(targets: &[Target]) {
    let Some(dir) = base_dir() else { return };
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let saved = Saved {
        saved_at: now_secs(),
        targets: targets.to_vec(),
    };
    if let Ok(json) = serde_json::to_vec(&saved) {
        let _ = std::fs::write(dir.join("last-scan.json"), json);
    }
}

/// Retorna (idade em minutos, alvos) do último resultado salvo.
pub fn load() -> Option<(u64, Vec<Target>)> {
    let bytes = std::fs::read(base_dir()?.join("last-scan.json")).ok()?;
    let s: Saved = serde_json::from_slice(&bytes).ok()?;
    Some((now_secs().saturating_sub(s.saved_at) / 60, s.targets))
}

/// Caminho do log de auditoria.
pub fn audit_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("scour").join("audit.log"))
}

/// Inicia o log (tracing) em arquivo: toda remoção é registrada. Retorna o caminho.
pub fn init_logging() -> Option<PathBuf> {
    let path = audit_path()?;
    std::fs::create_dir_all(path.parent()?).ok()?;
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    tracing_subscriber::fmt()
        .with_writer(Mutex::new(file))
        .with_ansi(false)
        .with_target(false)
        .try_init()
        .ok()?;
    Some(path)
}
