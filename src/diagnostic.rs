//! Diagnostic Module for PoshBuddy
//!
//! Provides system environment health checks, shell profile analysis,
//! syntax validation, and diagnostic reporting.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Errors that may occur during diagnostics
#[derive(Debug)]
#[allow(dead_code)]
pub enum DiagnosticError {
    Io(io::Error),
    #[allow(dead_code)]
    InvalidSyntax(String),
    #[allow(dead_code)]
    PowerShellNotFound,
    #[allow(dead_code)]
    ProfileNotReadable(String),
}

impl fmt::Display for DiagnosticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticError::Io(e) => write!(f, "I/O Error: {}", e),
            DiagnosticError::InvalidSyntax(msg) => write!(f, "Invalid syntax: {}", msg),
            DiagnosticError::PowerShellNotFound => write!(f, "PowerShell not found"),
            DiagnosticError::ProfileNotReadable(path) => {
                write!(f, "Profile is not readable: {}", path)
            }
        }
    }
}

impl std::error::Error for DiagnosticError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DiagnosticError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DiagnosticError {
    fn from(e: io::Error) -> Self {
        DiagnosticError::Io(e)
    }
}

/// Result of a diagnostic check
#[derive(Debug, Clone, PartialEq)]
pub struct DiagnosticResult {
    pub success: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

impl DiagnosticResult {
    pub fn new() -> Self {
        Self {
            success: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        self.success = false;
    }

    pub fn add_suggestion(&mut self, msg: impl Into<String>) {
        self.suggestions.push(msg.into());
    }

    pub fn is_valid(&self) -> bool {
        self.success && self.errors.is_empty()
    }
}

impl Default for DiagnosticResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Diagnostic engine for environment and profile checks
pub struct Diagnostic;

impl Diagnostic {
    pub fn new() -> Self {
        Self
    }

    /// Verifies if Oh My Posh is installed in system PATH
    pub fn check_oh_my_posh() -> bool {
        let output = if cfg!(windows) {
            Command::new("cmd")
                .args(["/C", "where oh-my-posh"])
                .output()
        } else {
            Command::new("which").arg("oh-my-posh").output()
        };

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    /// Verifies if PowerShell is available
    pub fn is_powershell_available() -> bool {
        let cmd = if cfg!(windows) { "powershell" } else { "pwsh" };
        Command::new(cmd)
            .arg("-Command")
            .arg("$PSVersionTable.PSVersion")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Verifies the syntax of a shell / PowerShell script
    pub fn validate_script_syntax(&self, script: &str) -> DiagnosticResult {
        let mut result = DiagnosticResult::new();

        if script.trim().is_empty() {
            result.add_error("Script content is empty");
            return result;
        }

        let open_braces = script.chars().filter(|&c| c == '{').count();
        let close_braces = script.chars().filter(|&c| c == '}').count();
        if open_braces != close_braces {
            result.add_error(format!(
                "Unbalanced braces: {} open, {} closed",
                open_braces, close_braces
            ));
        }

        let open_parens = script.chars().filter(|&c| c == '(').count();
        let close_parens = script.chars().filter(|&c| c == ')').count();
        if open_parens != close_parens {
            result.add_error(format!(
                "Unbalanced parentheses: {} open, {} closed",
                open_parens, close_parens
            ));
        }

        let double_quotes = script.chars().filter(|&c| c == '"').count();
        let single_quotes = script.chars().filter(|&c| c == '\'').count();
        if double_quotes % 2 != 0 {
            result.add_warning("Possible unbalanced double quotes");
        }
        if single_quotes % 2 != 0 {
            result.add_warning("Possible unbalanced single quotes");
        }

        if let Some(config_idx) = script.find("--config") {
            let after_config = &script[config_idx + 8..];
            if let Some(quote_idx) = after_config.find(['"', '\'']) {
                let quote_char = after_config.chars().nth(quote_idx).unwrap();
                let path_end = after_config[quote_idx + 1..].find(quote_char).unwrap_or(0);
                if path_end > 0 {
                    let theme_path = &after_config[quote_idx + 1..quote_idx + 1 + path_end];
                    if !Path::new(theme_path).exists() {
                        result.add_warning(format!(
                            "Referenced theme path does not exist: {}",
                            theme_path
                        ));
                    }
                }
            }
        }

        if script.contains("Invoke-Expression") {
            result.add_suggestion("Ensure Invoke-Expression is used with trusted sources");
        }

        result
    }

    /// Verifies a specific shell profile file
    pub fn check_profile(&self, profile_path: &Path) -> Result<DiagnosticResult, DiagnosticError> {
        let mut result = DiagnosticResult::new();

        if !profile_path.exists() {
            result.add_warning(format!(
                "Profile does not exist: {}",
                profile_path.display()
            ));
            return Ok(result);
        }

        match std::fs::read_to_string(profile_path) {
            Ok(content) => {
                let syntax_result = self.validate_script_syntax(&content);
                result.warnings.extend(syntax_result.warnings);
                result.errors.extend(syntax_result.errors);
                result.suggestions.extend(syntax_result.suggestions);
                result.success = result.errors.is_empty();

                let bytes = std::fs::read(profile_path)?;
                if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                    result.add_suggestion("Profile contains UTF-8 BOM encoding");
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(profile_path) {
                        let permissions = metadata.permissions();
                        let mode = permissions.mode();
                        if mode & 0o111 == 0 {
                            result.add_warning("Profile does not have execution permissions");
                        }
                    }
                }
            }
            Err(e) => {
                result.add_error(format!(
                    "Could not read profile {}: {}",
                    profile_path.display(),
                    e
                ));
            }
        }

        Ok(result)
    }

    /// Runs complete system diagnostic check
    pub fn run_full_diagnostics(
        &self,
        profiles: &[PathBuf],
        active_config_path: Option<&PathBuf>,
        has_nerd_font: bool,
        shell_name: &str,
        terminal_name: &str,
    ) -> DiagnosticResult {
        let mut result = DiagnosticResult::new();

        // 1. Oh My Posh Binary Check
        if Self::check_oh_my_posh() {
            result.add_suggestion("Oh My Posh executable detected in PATH");
        } else {
            result.add_warning("Oh My Posh binary was not found in PATH");
        }

        // 2. PowerShell / Shell Check
        if Self::is_powershell_available() {
            result.add_suggestion(format!("PowerShell environment available ({})", shell_name));
        } else {
            result.add_suggestion(format!("Running under shell: {}", shell_name));
        }

        // 3. Terminal & Nerd Font Check
        if has_nerd_font {
            result.add_suggestion(format!(
                "Nerd Font detected in terminal ({})",
                terminal_name
            ));
        } else {
            result.add_warning(format!(
                "Nerd Font missing or unconfirmed in terminal ({})",
                terminal_name
            ));
        }

        // 4. Active Theme Configuration Check
        if let Some(config_path) = active_config_path {
            if config_path.exists() {
                result.add_suggestion(format!(
                    "Active theme config present: {}",
                    config_path.display()
                ));
            } else {
                result.add_error(format!(
                    "Active theme config path set but file does not exist: {}",
                    config_path.display()
                ));
            }
        } else {
            result.add_warning("No active Oh My Posh configuration file detected");
        }

        // 5. Detected Shell Profiles Check
        if profiles.is_empty() {
            result.add_warning("No shell configuration profiles detected on system");
        } else {
            for profile in profiles {
                match self.check_profile(profile) {
                    Ok(prof_res) => {
                        result.warnings.extend(prof_res.warnings);
                        result.errors.extend(prof_res.errors);
                        result.suggestions.extend(prof_res.suggestions);
                    }
                    Err(e) => {
                        result.add_error(format!(
                            "Failed to diagnose profile {}: {}",
                            profile.display(),
                            e
                        ));
                    }
                }
            }
        }

        result.success = result.errors.is_empty();
        result
    }

    /// Formats diagnostic result into a readable report string
    pub fn format_report(&self, result: &DiagnosticResult) -> String {
        let mut report = String::new();
        report.push_str("═══════════════════════════════════════════\n");
        report.push_str("          SYSTEM DIAGNOSTIC REPORT\n");
        report.push_str("═══════════════════════════════════════════\n\n");

        if result.is_valid() {
            report.push_str("✅ Status: ALL SYSTEMS OPERATIONAL\n");
        } else {
            report.push_str("❌ Status: ISSUES DETECTED\n");
        }

        if !result.errors.is_empty() {
            report.push_str("\n🚨 ERRORS:\n");
            for error in &result.errors {
                report.push_str(&format!("   • {}\n", error));
            }
        }

        if !result.warnings.is_empty() {
            report.push_str("\n⚠️  WARNINGS:\n");
            for warning in &result.warnings {
                report.push_str(&format!("   • {}\n", warning));
            }
        }

        if !result.suggestions.is_empty() {
            report.push_str("\n💡 SYSTEM CHECKS & SUGGESTIONS:\n");
            for suggestion in &result.suggestions {
                report.push_str(&format!("   • {}\n", suggestion));
            }
        }

        report.push_str("\n═══════════════════════════════════════════");
        report
    }
}

impl Default for Diagnostic {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_script_empty() {
        let diag = Diagnostic::new();
        let result = diag.validate_script_syntax("");
        assert!(!result.is_valid());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validate_script_balanced() {
        let diag = Diagnostic::new();
        let result = diag.validate_script_syntax("function test() { Write-Host 'ok' }");
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_script_unbalanced_braces() {
        let diag = Diagnostic::new();
        let result = diag.validate_script_syntax("function test() { Write-Host 'ok'");
        assert!(!result.is_valid());
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("Unbalanced braces"))
        );
    }

    #[test]
    fn test_diagnostic_result_methods() {
        let mut result = DiagnosticResult::new();
        assert!(result.is_valid());

        result.add_warning("Test warning");
        assert!(result.is_valid());

        result.add_error("Test error");
        assert!(!result.is_valid());
    }

    #[test]
    fn test_format_report() {
        let diag = Diagnostic::new();
        let mut result = DiagnosticResult::new();
        result.add_error("Test error");
        result.add_warning("Test warning");
        result.add_suggestion("Test suggestion");

        let report = diag.format_report(&result);
        assert!(report.contains("ISSUES DETECTED"));
        assert!(report.contains("Test error"));
        assert!(report.contains("Test warning"));
        assert!(report.contains("Test suggestion"));
    }

    #[test]
    fn test_check_profile_nonexistent() {
        let diag = Diagnostic::new();
        let path = Path::new("nonexistent_profile.ps1");
        let result = diag.check_profile(path).unwrap();

        assert!(result.is_valid());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("Profile does not exist"))
        );
    }

    #[test]
    fn test_check_profile_valid() {
        use std::io::Write;
        let diag = Diagnostic::new();
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "Write-Host 'Hello'").unwrap();

        let result = diag.check_profile(temp_file.path()).unwrap();
        assert!(result.is_valid());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_run_full_diagnostics() {
        let diag = Diagnostic::new();
        let profiles = vec![];
        let active_config = None;

        let result =
            diag.run_full_diagnostics(&profiles, active_config, true, "zsh", "xterm-256color");
        assert!(!result.suggestions.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.contains("No active Oh My Posh configuration"))
        );
    }
}
