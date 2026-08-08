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

     #[arg(long = "check")]
    pub check: bool,

    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    #[arg(raw = true, trailing_var_arg = true)]
    pub extra_args: Vec<String>,
}

impl Args {
    pub fn has_check_flag(&self) -> bool {
    self.check || self.extra_args.iter().any(|arg| arg == "--check")
}

    pub fn get_filtered_args(&self) -> Vec<String> {
     let mut filtered = Vec::new();
     let mut skip_next = false;
     for arg in &self.extra_args {
         if skip_next {
             skip_next = false;
             continue;
         }
         if arg == "--check" {
             continue;
         } 
         if arg == "--model" {
             skip_next = true;
             continue;
         }
         filtered.push(arg.clone());
     }
     filtered
}
}