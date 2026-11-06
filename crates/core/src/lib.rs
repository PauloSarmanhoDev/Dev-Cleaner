//! Núcleo do dev-cleaner, compartilhado pela CLI e pela interface Tauri.
//! Não contém nenhuma UI: o progresso é reportado por callbacks.

pub mod cleaner;
pub mod discovery;
pub mod model;
pub mod scanner;

use std::sync::Mutex;

/// Avisos não fatais (permissão negada, arquivo bloqueado...) coletados entre threads.
pub type Warnings = Mutex<Vec<String>>;

