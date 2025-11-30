use crate::utils::ensure_dir;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Résultat de l'exécution d'un script Python.
pub struct CodeExecutionResult {
    pub script_path: PathBuf,
    pub stdout: String,
    pub stderr: String,
}

/// Responsable de l'écriture des scripts Python sur le disque et de leur exécution.
pub struct CodeExecutor {
    base_dir: PathBuf,
}

impl CodeExecutor {
    /// Crée un exécuteur de code.
    ///
    /// `base_dir` : répertoire où seront stockés les scripts générés.
    pub fn new(base_dir: &str) -> Result<Self> {
        let dir = PathBuf::from(base_dir);
        ensure_dir(&dir)?;
        Ok(Self { base_dir: dir })
    }

    /// Écrit un script Python dans un fichier et l'exécute avec l'interpréteur `python` ou `python3`.
    ///
    /// Attention : ce code exécute du Python généré automatiquement.
    /// À n'utiliser que dans un environnement de test contrôlé.
    pub fn write_and_run(&self, code: &str) -> Result<CodeExecutionResult> {
        // Nom de fichier basé sur un timestamp pour éviter les collisions.
        let ts = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("script_{ts}.py");
        let script_path = self.base_dir.join(filename);

        fs::write(&script_path, code)
            .with_context(|| format!("Impossible d'écrire le script {:?}", script_path))?;

        // On essaie d'abord `python3`, puis `python` si besoin.
        let python_cmds = ["python3", "python"];

        let mut last_err: Option<anyhow::Error> = None;

        for cmd in python_cmds {
            let output = Command::new(cmd)
                .arg(&script_path)
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    return Ok(CodeExecutionResult {
                        script_path,
                        stdout,
                        stderr,
                    });
                }
                Err(e) => {
                    last_err = Some(anyhow::anyhow!(
                        "Échec avec la commande `{cmd}` : {e}"
                    ));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!(
            "Impossible d'exécuter le script avec python/python3"
        )))
    }
}
