use colored::*;
use tempfile::NamedTempFile;
use regex::Regex;
use std::io::Write;
use anyhow::Result;
use crate::aur::AurClient;

pub struct Scanner;

impl Scanner {
    pub fn new() -> Self {
        Self
    }

    pub fn scan_package(&self, package: &str) -> Result<Option<Vec<String>>> {
        let content = AurClient::fetch_pkgbuild(package)?;
        
        let mut temp_file = NamedTempFile::new()
            .expect("Failed to create temporary file");
        temp_file.write_all(content.as_bytes())
            .expect("Failed to write PKGBUILD");
        
        if let Some(path) = temp_file.path().to_str() {
            println!("{} {}", "PKGBUILD saved to".bright_blue(), path);
        }

        Ok(self.analyze_heuristic(&content))
    }

    pub fn analyze_heuristic(&self, content: &str) -> Option<Vec<String>> {
        let mut warnings = Vec::new();

        if content.contains("curl") && content.contains("| sh") {
            warnings.push("Downloads and executes script from remote source".to_string());
        }
        
        if content.contains("wget") && content.contains("| sh") {
            warnings.push("Downloads and executes script from remote source".to_string());
        }
        
        if content.contains("sudo") {
            warnings.push("Uses sudo, may escalate privileges".to_string());
        }
        
        if content.contains("chmod 777") {
            warnings.push("Sets overly permissive file permissions".to_string());
        }
        
        if content.contains("rm -rf /") {
            warnings.push("Dangerous deletion command".to_string());
        }
        
        if content.contains("python -c") {
            warnings.push("Python code execution, could be obfuscated".to_string());
        }
        
        if content.contains("base64 -d") {
            warnings.push("Base64 decoding, could hide malicious code".to_string());
        }
        
        if content.contains("eval $") {
            warnings.push("Variable evaluation, could be used for code injection".to_string());
        }
        
        if content.contains("systemd-run") {
            warnings.push("Systemd service injection".to_string());
        }
        
        if content.contains("crontab") {
            warnings.push("Cron job installation".to_string());
        }

        if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        }
    }

    pub fn print_warnings(&self, warnings: &[String]) {
        println!("{}", "Potential issues detected:".yellow().bold());
        for warning in warnings {
            println!("   {}", warning.red());
        }
    }

    pub fn confirm_continue(&self) -> bool {
        print!("Continue anyway? (y/N): ");
        let _ = std::io::stdout().flush();
        
        let mut response = String::new();
        let _ = std::io::stdin().read_line(&mut response);
        
        let response = response.trim().to_lowercase();
        response == "y" || response == "yes"
    }
}
