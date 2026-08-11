mod args;
mod aur;
mod scanner;
mod yay;
mod llm;
mod config;

use clap::Parser;
use colored::Colorize;
use anyhow::Result;
use args::Args;
use scanner::Scanner;
use yay::YayWrapper;
use std::path::PathBuf;
use config::Config;

fn main() -> Result<()> {
    let config = Config::load()?;
    let args = Args::parse();
    
    print_banner(&config);

    if !YayWrapper::is_installed() {
        YayWrapper::show_install_instructions();
        std::process::exit(1);
    }

    YayWrapper::warn_about_paru();

    // Check if we should scan
    let should_scan = if args.no_scan {
        false
    } else if args.install.is_some() {
        config.scan_by_default
    } else {
        false
    };

    if should_scan && args.install.is_some() {
        let package_name = args.install.as_ref().unwrap();
        
        if !args.quiet {
            if config.color {
                println!("{} {}", "Scanning".bright_blue(), package_name.bright_white());
            } else {
                println!("Scanning {}", package_name);
            }
        }

        let model_path = args.model_path.clone().map(PathBuf::from);
        let scanner = Scanner::new(!args.no_llm, model_path, config.clone());

        match scanner.scan_package(package_name) {
            Ok(Some(warnings)) => {
                if !args.quiet {
                    scanner.print_warnings(&warnings);
                }
                
                if !scanner.confirm_continue() {
                    if config.color {
                        println!("{}", "Installation cancelled.".bright_red());
                    } else {
                        println!("Installation cancelled.");
                    }
                    std::process::exit(1);
                }
            }
            Ok(None) => {
                if !args.quiet {
                    if config.color {
                        println!("{}", "No malware detected - Safe to proceed.".bright_green());
                    } else {
                        println!("No malware detected - Safe to proceed.");
                    }
                }
            }
            Err(e) => {
                if config.color {
                    eprintln!("{}", format!("Scan failed: {}", e).yellow());
                } else {
                    eprintln!("Scan failed: {}", e);
                }
                if !scanner.confirm_continue() {
                    if config.color {
                        println!("{}", "Installation cancelled.".bright_red());
                    } else {
                        println!("Installation cancelled.");
                    }
                    std::process::exit(1);
                }
            }
        }

        YayWrapper::install_package(package_name, &args)?;
    } else {
        if !args.quiet && args.install.is_some() && args.no_scan {
            if config.color {
                println!("{}", "Skipping malware scan (--no-scan flag used)".bright_yellow());
            } else {
                println!("Skipping malware scan (--no-scan flag used)");
            }
        }
        YayWrapper::execute_with_args(&args)?;
    }

    Ok(())
}

fn print_banner(config: &Config) {
    if config.color {
        println!("{}", "AUREK - AUR Security Checker".bright_cyan().bold());
        println!("{}", "Checking packages before installation...".dimmed());
    } else {
        println!("AUREK - AUR Security Checker");
        println!("Checking packages before installation...");
    }
    println!();
}