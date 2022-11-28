use anyhow::Context;
use clap::Parser;
use log::LevelFilter;
use std::fs;

mod api;
mod models;
mod player;
mod storage;
mod ui;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opt {
    /// Logging level
    #[clap(long, default_value = "error")]
    log_level: LevelFilter,

    /// Log file path (for debugging)
    #[clap(long)]
    log_filepath: Option<String>,

    /// SQLite database path
    #[clap(long)]
    db_filepath: Option<String>,
}

impl Opt {
    fn default_filepath(name: &str) -> String {
        let dir = dirs::config_dir()
            .expect("failed to find os config dir")
            .join("tradio");

        fs::create_dir_all(&dir).expect("failed to create dir");
        dir.join(name).to_str().unwrap().to_string()
    }

    fn log_filepath(&self) -> String {
        if let Some(ref path) = self.log_filepath {
            return path.clone();
        }

        Self::default_filepath("tradio.log")
    }

    fn db_filepath(&self) -> String {
        if let Some(ref path) = self.log_filepath {
            return path.clone();
        }

        Self::default_filepath("sqlite.db")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let log_file = fs::File::create(&opt.log_filepath()).context("can't open log file")?;

    simplelog::WriteLogger::init(opt.log_level, simplelog::Config::default(), log_file)
        .context("init logger")?;

    let player = player::Rodio::default()?;
    let storage = storage::Sqlite::new(&opt.db_filepath()).await?;

    ui::Ui::new(player, storage)
        .with_client(Box::new(api::RadioBrowser::new()))
        .start()
        .await
}
