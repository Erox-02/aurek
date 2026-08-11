use colored::*;
use std::process::{Command, exit};
use anyhow::{Result, Context};
use which::which;
use crate::args::Args;

pub struct YayWrapper;

impl YayWrapper {
    pub fn is_installed() -> bool {
        which("yay").is_ok()
    }

    pub fn is_paru_installed() -> bool {
        which("paru").is_ok()
    }

    pub fn warn_about_paru() {
        if Self::is_paru_installed() && !Self::is_installed() {
            println!("\n{}", "WARNING: You're using paru.".yellow().bold());
            println!("{}", "yay is the correct AUR helper.".bright_blue());
            println!("   https://github.com/Jguer/yay\n");
        }
    }

    pub fn show_install_instructions() {
        eprintln!("{}", "❌ Error: yay is not installed. Please install yay first.".bright_red()); //add auto yay installation
        eprintln!("   Visit: https://github.com/Jguer/yay");
    }

    pub fn install_package(package: &str, args: &Args) -> Result<()> {
        let filtered_args = args.get_filtered_args();
        let mut cmd = Command::new("yay");
        cmd.arg("-S").arg(package);
        cmd.args(&filtered_args);

        if !args.quiet {
            println!("{} {}", "Executing:".bright_blue(), 
                     format!("yay -S {} {}", package, filtered_args.join(" ")).bright_white());
        }

        let status = cmd.status()
            .context("Failed to execute yay command")?;

        if !status.success() {
            exit(status.code().unwrap_or(1));
        }

        Ok(())
    }

    pub fn execute_with_args(args: &Args) -> Result<()> {
        let mut cmd = Command::new("yay");
        
        if let Some(pkg) = &args.install {
            cmd.arg("-S").arg(pkg);
        }

        if let Some(pkg) = &args.remove {
            cmd.arg("-R").arg(pkg);
        }
        
        cmd.args(&args.extra_args);
        
        let status = cmd.status()
            .context("Failed to execute yay command")?;

        if !status.success() {
            exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}