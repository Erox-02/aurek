use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "aurek")]
#[command(author = "Dipanjan Dutta")]
#[command(version = "0.2.0")]
#[command(about = "AUR security checker with LLM analysis")]
pub struct Args {

    #[arg(short = 'S', long = "install")]
    pub install: Option<String>,

    #[arg(long = "no-scan")]
    pub no_scan: bool,

    #[arg(long = "no-llm")]
    pub no_llm: bool,

    #[arg(long = "model")]
    pub model_path: Option<String>,

    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    #[arg(raw = true, trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

impl Args {
    pub fn has_check_flag(&self) -> bool {
        self.extra_args.iter().any(|arg| arg == "--check")
    }

    pub fn get_filtered_args(&self) -> Vec<String> {
        self.extra_args
            .iter()
            .filter(|arg| *arg != "--check")
            .cloned()
            .collect()
    }
}