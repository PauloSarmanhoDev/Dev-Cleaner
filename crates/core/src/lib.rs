//! Núcleo do dev-cleaner, compartilhado pela CLI e pela interface Tauri.
//! Não contém nenhuma UI: o progresso é reportado por callbacks.

pub mod cleaner;
pub mod config;
pub mod discovery;
pub mod i18n;
pub mod model;
pub mod pipeline;
pub mod procs;
pub mod safety;
pub mod scanner;
pub mod store;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Avisos não fatais (permissão negada, arquivo bloqueado...) coletados entre threads.
pub type Warnings = Mutex<Vec<String>>;

/// Sinalizador de cancelamento compartilhável entre threads.
#[derive(Debug, Clone, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn reset(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}
