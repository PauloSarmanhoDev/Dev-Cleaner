//! Detecção de IDEs em execução (arquivos de cache costumam ficar bloqueados).

use crate::model::Target;
use std::collections::HashSet;
use sysinfo::{ProcessesToUpdate, System};

/// "Code.exe" → "code", "idea64.exe" → "idea".
fn normalize(name: &str) -> String {
    let n = name.to_lowercase();
    let n = n.strip_suffix(".exe").unwrap_or(&n);
    n.strip_suffix("64").unwrap_or(n).to_string()
}

/// Marca em `running` os alvos cuja IDE está aberta.
pub fn annotate(targets: &mut [Target]) {
    for t in targets.iter_mut() {
        t.running = None;
    }
    if targets.iter().all(|t| t.apps.is_empty()) {
        return;
    }
    // Se o sistema não permitir listar processos, simplesmente não anotamos nada.
    let Ok(mut sys) = System::new() else { return };
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let names: HashSet<String> = sys
        .processes()
        .values()
        .map(|p| normalize(&p.name().to_string_lossy()))
        .collect();
    for t in targets.iter_mut() {
        t.running = t.apps.iter().find(|a| names.contains(*a)).cloned();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_process_names() {
        assert_eq!(normalize("Code.exe"), "code");
        assert_eq!(normalize("idea64.exe"), "idea");
        assert_eq!(normalize("studio64.exe"), "studio");
        assert_eq!(normalize("zed"), "zed");
    }
}
