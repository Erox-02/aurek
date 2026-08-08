use colored::*;
use tempfile::NamedTempFile;
use std::io::Write;
use anyhow::Result;
use crate::aur::AurClient;
use crate::llm::LocalLLM;
use std::path::PathBuf;

pub struct Scanner {
    use_llm: bool,
    model_path: Option<PathBuf>,
}

impl Scanner {
    pub fn new(use_llm: bool, model_path: Option<PathBuf>) -> Self {
        Self {
            use_llm,
            model_path,
        }
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

        let mut warnings = Vec::new();

        if self.use_llm {
            match LocalLLM::new(self.model_path.clone()) {
                Ok(mut llm) => {
                    println!("{}", "🧠 Running local LLM analysis with Gemma...".bright_cyan());
                    
                    match llm.start_server() {
                        Ok(()) => {
                            match llm.analyze_pkgbuild(&content, package) {
                                Ok(llm_warnings) => {
                                    if !llm_warnings.is_empty() {
                                        warnings.extend(llm_warnings);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{} {}", "".yellow(), format!("LLM analysis failed: {}", e).yellow());
                                    eprintln!("   Falling back to heuristic analysis...");
                                    warnings.extend(self.heuristic_analysis(&content));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "".yellow(), format!("Failed to start LLM server: {}", e).yellow());
                            eprintln!("   Falling back to heuristic analysis...");
                            warnings.extend(self.heuristic_analysis(&content));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "".yellow(), format!("LLM not available: {}", e).yellow());
                    eprintln!("   Using heuristic analysis instead...");
                    warnings.extend(self.heuristic_analysis(&content));
                }
            }
        } else {
            warnings.extend(self.heuristic_analysis(&content));
        }

        if warnings.is_empty() {
            Ok(None)
        } else {
            Ok(Some(warnings))
        }
    }

    fn heuristic_analysis(&self, content: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        if content.contains("curl") && content.contains("| sh") {
            warnings.push("Remote Code Execution: Downloads and executes script from remote source".to_string());
        }
        
        if content.contains("wget") && content.contains("| sh") {
            warnings.push("Remote Code Execution: Downloads and executes script from remote source".to_string());
        }
        
        if content.contains("sudo") {
            warnings.push("Privilege Escalation: Uses sudo, may escalate privileges".to_string());
        }
        
        if content.contains("chmod 777") {
            warnings.push("Permission Issue: Sets overly permissive file permissions".to_string());
        }
        
        if content.contains("rm -rf /") {
            warnings.push("Destructive: Dangerous deletion command".to_string());
        }
        
        if content.contains("python -c") {
            warnings.push("Code Obfuscation: Python code execution, could be obfuscated".to_string());
        }
        
        if content.contains("base64 -d") {
            warnings.push("Code Obfuscation: Base64 decoding, could hide malicious code".to_string());
        }
        
        if content.contains("eval $") {
            warnings.push("Code Injection: Variable evaluation, could be used for code injection".to_string());
        }
        
        if content.contains("systemd-run") {
            warnings.push("System Persistence: Systemd service injection".to_string());
        }
        
        if content.contains("crontab") {
            warnings.push("System Persistence: Cron job installation".to_string());
        }

        warnings
    }

    pub fn print_warnings(&self, warnings: &[String]) {
        println!("{}", " Potential issues detected:".yellow().bold());
        for warning in warnings {
            println!("   {}", warning.red());
        }
    }

    pub fn confirm_continue(&self) -> bool {
        print!("{} ", "Continue anyway? (y/N):".bright_yellow());
        let _ = std::io::stdout().flush();
        
        let mut response = String::new();
        let _ = std::io::stdin().read_line(&mut response);
        
        let response = response.trim().to_lowercase();
        response == "y" || response == "yes"
    }
}