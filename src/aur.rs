use anyhow::{anyhow, Context, Result};
use reqwest;
use serde_json::Value;

pub struct AurClient;

impl AurClient {
    pub fn fetch_pkgbuild(package: &str) -> Result<String> {
        let url = format!(
            "https://aur.archlinux.org/rpc/v5/info?arg[]={}",
            package
        );

        let response = reqwest::blocking::get(&url)
            .context("Failed to connect to AUR API")?;

        let json: Value = response
            .json()
            .context("Failed to parse AUR API response")?;

        let results = json
            .get("results")
            .and_then(|r| r.as_array())
            .ok_or_else(|| anyhow!("No results found for package: {}", package))?;

        if results.is_empty() {
            return Err(anyhow!("Package not found in AUR: {}", package));
        }

        let pkgbase = results[0]
            .get("PackageBase")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing PackageBase field"))?;

        let pkgbuild_url = format!(
            "https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h={}",
            pkgbase
        );

        let response = reqwest::blocking::get(&pkgbuild_url)
            .context("Failed to fetch PKGBUILD")?;

        Ok(response
            .text()
            .context("Failed to read PKGBUILD content")?)
    }
}
