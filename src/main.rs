mod args;
mod aur;
mod scanner;
mod yay;
mod llm;

use clap::Parser;
use anyhow::Result;
use args::Args;
use scanner::Scanner;
use yay::YayWrapper;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse();
    
    print_banner();

    if !YayWrapper::is_installed() {
        YayWrapper::show_install_instructions();
        std::process::exit(1);
    }

    YayWrapper::warn_about_paru();

    if args.has_check_flag() && args.install.is_some() {
        let package_name = args.install.as_ref().unwrap();
        
        if !args.no_scan {
            if !args.quiet {
                println!("{} {}", "Scanning", package_name);
            }

            let model_path = args.model_path.clone().map(PathBuf::from);

            let scanner = Scanner::new(!args.no_llm, model_path);
            match scanner.scan_package(package_name) {
                Ok(Some(warnings)) => {
                    if !args.quiet {
                        scanner.print_warnings(&warnings);
                    }
                    
                    if !scanner.confirm_continue() {
                        println!("{}", "Installation cancelled.");
                        std::process::exit(1);
                    }
                }
                Ok(None) => {
                    if !args.quiet {
                        println!("{}", "No malware detected - Safe to proceed.");
                    }
                }
                Err(e) => {
                    eprintln!("{}", format!("Scan failed: {}", e));
                    if !scanner.confirm_continue() {
                        println!("{}", "Installation cancelled.");
                        std::process::exit(1);
                    }
                }
            }
        } else {
            if !args.quiet {
                println!("{}", "Skipping malware scan (--no-scan flag used)");
            }
        }

        YayWrapper::install_package(package_name, &args)?;
    } else {
        YayWrapper::execute_with_args(&args)?;
    }

    Ok(())
}

fn print_banner() {
    println!("{}", "Checking packages before installation...");
    println!();
}