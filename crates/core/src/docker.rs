//! Imagens e volumes do Docker. Diferente dos demais alvos, não são pastas que dá para
//! apagar: a remoção passa pelo próprio Docker (`image prune` / `volume prune`).

use crate::model::{Category, Target, TargetKind};
use std::path::PathBuf;
use std::process::Command;

fn docker(args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = Command::new("docker");
    cmd.args(args);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd.output()
}

/// "1.2GB (45%)" → bytes. O Docker usa unidades decimais (kB, MB, GB, TB).
fn parse_size(s: &str) -> Option<u64> {
    let first = s.split_whitespace().next()?;
    let idx = first.find(|c: char| c.is_ascii_alphabetic())?;
    let (num, unit) = first.split_at(idx);
    let n: f64 = num.parse().ok()?;
    let mult = match unit.to_ascii_lowercase().as_str() {
        "b" => 1.0,
        "kb" => 1e3,
        "mb" => 1e6,
        "gb" => 1e9,
        "tb" => 1e12,
        _ => return None,
    };
    Some((n * mult) as u64)
}

/// Interpreta a saída de `docker system df --format "{{json .}}"` (uma linha JSON por tipo).
fn parse_df(out: &str) -> Vec<Target> {
    let mut t = Vec::new();
    for line in out.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let size = v["Reclaimable"]
            .as_str()
            .and_then(parse_size)
            .unwrap_or_default();
        let (kind, name, id, critical) = match v["Type"].as_str() {
            Some("Images") => (
                TargetKind::DockerImages,
                "Docker images (unused)",
                "docker://images",
                false,
            ),
            // Volumes nomeados costumam guardar dados (bancos etc.): crítico.
            Some("Local Volumes") => (
                TargetKind::DockerVolumes,
                "Docker volumes (unused)",
                "docker://volumes",
                true,
            ),
            _ => continue,
        };
        if size > 0 {
            let mut tg =
                Target::new(Category::Tool, name, PathBuf::from(id), critical).with_kind(kind);
            tg.size = size;
            t.push(tg);
        }
    }
    t
}

/// Alvos Docker com o espaço recuperável. Vazio se o Docker não estiver instalado/ativo.
pub fn targets() -> Vec<Target> {
    match docker(&["system", "df", "--format", "{{json .}}"]) {
        Ok(o) if o.status.success() => parse_df(&String::from_utf8_lossy(&o.stdout)),
        _ => Vec::new(),
    }
}

/// Executa a limpeza do alvo Docker. Devolve a mensagem de erro, se houver.
pub fn prune(kind: TargetKind) -> Result<(), String> {
    let args: &[&str] = match kind {
        TargetKind::DockerImages => &["image", "prune", "-a", "-f"],
        TargetKind::DockerVolumes => &["volume", "prune", "-a", "-f"],
        TargetKind::Dir => return Ok(()),
    };
    match docker(args) {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_sizes() {
        assert_eq!(parse_size("1.5GB (45%)"), Some(1_500_000_000));
        assert_eq!(parse_size("512kB"), Some(512_000));
        assert_eq!(parse_size("0B"), Some(0));
        assert_eq!(parse_size("abc"), None);
    }

    #[test]
    fn parses_df_output() {
        let out = r#"{"Type":"Images","TotalCount":"5","Size":"3GB","Reclaimable":"2GB (66%)"}
{"Type":"Containers","Reclaimable":"1GB"}
{"Type":"Local Volumes","Reclaimable":"100MB (10%)"}
{"Type":"Build Cache","Reclaimable":"0B"}"#;
        let t = parse_df(out);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].kind, TargetKind::DockerImages);
        assert_eq!(t[0].size, 2_000_000_000);
        assert!(!t[0].critical);
        assert_eq!(t[1].kind, TargetKind::DockerVolumes);
        assert!(t[1].critical);
    }
}
