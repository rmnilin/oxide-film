#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub home: Option<std::path::PathBuf>,
    #[arg(short, long)]
    pub init_file: Option<std::path::PathBuf>,
}
