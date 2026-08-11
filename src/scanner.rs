use tempfile::NamedTempFile;
use std::io::Write;
use anyhow::Result;
use crate::aur::AurClient;
use crate::llm::LLMClient;
use std::path::PathBuf;
use crate::config::Config;

pub struct Scanner {
    use_llm: bool,
    model_path: Option<PathBuf>,
    config: Config,
}

impl Scanner {
    pub fn new(use_llm: bool, model_path: Option<PathBuf>, config: Config) -> Self {
        Self {
            use_llm,
            model_path,
            config,
        }
    }

    pub fn scan_package(&self, package: &str) -> Result<Option<Vec<(String, String)>>> {
        let content = AurClient::fetch_pkgbuild(package)?;
        
        let mut temp_file = NamedTempFile::new()
            .expect("Failed to create temporary file");
        temp_file.write_all(content.as_bytes())
            .expect("Failed to write PKGBUILD");
        
        if let Some(path) = temp_file.path().to_str() {
            println!("PKGBUILD saved to {}", path);
        }

        let mut warnings: Vec<(String, String)> = Vec::new();

        if self.use_llm {
            match LLMClient::new(self.model_path.clone()) {
                Ok(mut llm) => {
                    println!("Running local LLM analysis with Gemma...");
                    
                    match llm.start_server() {
                        Ok(()) => {
                            match llm.analyze_pkgbuild(&content, package) {
                                Ok(llm_warnings) => {
                                    if !llm_warnings.is_empty() {
                                        for w in llm_warnings {
                                            if let Some((cat, desc)) = w.split_once(':') {
                                                warnings.push((cat.trim().to_string(), desc.trim().to_string()));
                                            } else {
                                                warnings.push(("General".to_string(), w));
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("LLM analysis failed: {}", e);
                                    eprintln!("Falling back to heuristic analysis...");
                                    warnings.extend(self.heuristic_analysis(&content));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to start LLM server: {}", e);
                            eprintln!("Falling back to heuristic analysis...");
                            warnings.extend(self.heuristic_analysis(&content));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("LLM not available: {}", e);
                    eprintln!("Using heuristic analysis instead...");
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

    fn heuristic_analysis(&self, content: &str) -> Vec<(String, String)> {
        let mut warnings = Vec::new();

        if content.contains("curl") && content.contains("| sh") {
            warnings.push(("Remote Code Execution".to_string(), "Downloads and executes script from remote source".to_string()));
        }

        if content.contains("wget") && content.contains("| sh") {
            warnings.push(("Remote Code Execution".to_string(), "Downloads and executes script from remote source".to_string()));
        }

        if content.contains("sudo") {
            warnings.push(("Privilege Escalation".to_string(), "Uses sudo, may escalate privileges".to_string()));
        }

        if content.contains("chmod 777") {
            warnings.push(("Permission Issue".to_string(), "Sets overly permissive file permissions".to_string()));
        }

        if content.contains("rm -rf /") {
            warnings.push(("Destructive".to_string(), "Dangerous deletion command".to_string()));
        }

        if content.contains("python -c") {
            warnings.push(("Code Obfuscation".to_string(), "Python code execution, could be obfuscated".to_string()));
        }

        if content.contains("base64 -d") {
            warnings.push(("Code Obfuscation".to_string(), "Base64 decoding, could hide malicious code".to_string()));
        }

        if content.contains("eval $") {
            warnings.push(("Code Injection".to_string(), "Variable evaluation, could be used for code injection".to_string()));
        }

        if content.contains("systemd-run") {
            warnings.push(("System Persistence".to_string(), "Systemd service injection".to_string()));
        }

        if content.contains("crontab") {
            warnings.push(("System Persistence".to_string(), "Cron job installation".to_string()));
        }

        warnings
    }

    pub fn print_warnings(&self, warnings: &[(String, String)]) {
        if warnings.is_empty() {
            println!("No issues detected");
            return;
        }

        println!("\nPotential issues detected:");
        println!();

        for (i, (category, description)) in warnings.iter().enumerate() {
            println!("  [#{}] {}:", i + 1, category);
            println!("      {}", description);
            println!();
        }

        println!("  AUREK recommends reviewing this package before installation.");
        println!();
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