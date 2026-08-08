use anyhow::{Result, Context, anyhow};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

pub struct LLMClient {
    server_url: String,
    model_path: PathBuf,
    server_process: Option<std::process::Child>,
}

impl LLMClient {
    pub fn new(model_path: Option<PathBuf>) -> Result<Self> {
        let model_path = model_path.unwrap_or_else(|| {
            let home = env::var("HOME").unwrap_or_default();
            PathBuf::from(format!("{}/models/gemma-2b-it-Q4_K_M.gguf", home))
        });
        
        if !model_path.exists() {
            return Err(anyhow!("Model path not found: {}", model_path.display()));
        }

        Ok(Self {
            server_url: "http://127.0.0.1:8080".to_string(),
            model_path,
            server_process: None,
        })
    }

    pub fn start_server(&mut self) -> Result<()> {
        // Check if server is already running and model is loaded
        if let Ok(response) = reqwest::blocking::get(&format!("{}/health", self.server_url)) {
            if response.status().is_success() {
                println!("llama-server already running");
                return Ok(());
            }
        }

        println!("Starting llama-server...");

        let server_path = which::which("llama-server")
            .context("llama.cpp not found")?;

        let child = Command::new(server_path)
            .arg("-m")
            .arg(&self.model_path)
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg("8080")
            .arg("-t")
            .arg("4")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("unable to start llama.cpp")?;

        self.server_process = Some(child);
        
        println!("Waiting for server to be ready...");
        // Wait longer and check for actual model loading
        for i in 0..60 {
            std::thread::sleep(Duration::from_secs(1));
            
            // Try to get health status
            if let Ok(response) = reqwest::blocking::get(&format!("{}/health", self.server_url)) {
                if response.status().is_success() {
                    // Try a simple completion to verify model is loaded
                    if let Ok(_) = self.test_completion() {
                        println!("Server ready!");
                        return Ok(());
                    }
                }
            }
            
            if i % 10 == 0 && i > 0 {
                println!("Still waiting... ({} seconds)", i);
            }
        }
        
        Err(anyhow!("Server failed to start within timeout"))
    }

    fn test_completion(&self) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let request_body = json!({
            "prompt": "Hello",
            "n_predict": 1,
            "temperature": 0.1,
        });

        let response = client
            .post(format!("{}/completion", self.server_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(Duration::from_secs(10))
            .send()
            .context("Failed to test completion")?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Model not ready"))
        }
    }

    pub fn analyze_pkgbuild(&self, content: &str, package_name: &str) -> Result<Vec<String>> {
        let prompt = format!(
            "Analyze this PKGBUILD file for malware or suspicious code.\n\
             Package: {}\n\
             PKGBUILD:\n\
             ```\n\
             {}\n\
             ```\n\n\
             Return a JSON array of security concerns. If none found, return an empty array [].\n\
             Format each concern as a string. Do NOT include your thinking process, only the response.",
            package_name, content
        );

        let response = self.call_llm(&prompt)?;
        self.parse_response(&response)
    }

    fn call_llm(&self, prompt: &str) -> Result<String> {
        let client = reqwest::blocking::Client::new();

        let request_body = json!({
            "prompt": prompt,
            "temperature": 0.1,
            "top_k": 10,
            "top_p": 0.95,
            "repeat_penalty": 1.1,
            "stop": ["```", "\n\n\n"],
        });

        let response = client
            .post(format!("{}/completion", self.server_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(Duration::from_secs(60))
            .send()
            .context("Failed to send request to llama.cpp server")?;

        if !response.status().is_success() {
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("llama-server error: {}", error_text));
        }

        let json_response: serde_json::Value = response
            .json()
            .context("Failed to parse server response")?;

        let content = json_response["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response format"))?
            .to_string();

        Ok(content)
    }

    fn parse_response(&self, response: &str) -> Result<Vec<String>> {
        // Strip out thinking process
        let response = if let Some(idx) = response.find("[End thinking]") {
            &response[idx + "[End thinking]".len()..]
        } else if let Some(idx) = response.find("response:") {
            &response[idx + "response:".len()..]
        } else {
            response
        };

        // Try to parse as JSON array first
        if let Ok(json_array) = serde_json::from_str::<Vec<String>>(response.trim()) {
            return Ok(json_array);
        }

        let mut warnings = Vec::new();
        for line in response.lines() {
            let line = line.trim();
            if line.starts_with('-') || line.starts_with('•') || line.starts_with('*') {
                let warning = line.trim_start_matches(|c| c == '-' || c == '•' || c == '*')
                    .trim()
                    .to_string();
                if !warning.is_empty() {
                    warnings.push(warning);
                }
            } else if line.contains(":") && !line.contains("```") && line.len() > 20 {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    warnings.push(format!("{}: {}", parts[0].trim(), parts[1].trim()));
                }
            }
        }

        if warnings.is_empty() && !response.is_empty() && !response.contains("[]") {
            if response.to_lowercase().contains("safe") || response.to_lowercase().contains("no issues") {
                return Ok(Vec::new());
            }
            warnings.push(response.trim().to_string());
        }

        Ok(warnings)
    }

    pub fn stop_server(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            let _ = child.kill();
            let _ = child.wait();
            println!("llama.cpp server stopped");
        }
    }
}

impl Drop for LLMClient {
    fn drop(&mut self) {
        self.stop_server();
    }
}