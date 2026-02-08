use clap::{Parser, Subcommand};
use self_update::backends::github;
use std::process;
use chrono::Local;
use env_logger::Builder;
use std::io::Write;

use grib2sail::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "grib2sail")]
#[command(about= "A cli for GRIB2Sail", long_about = None, version)]
struct Cli {
    #[arg(long, short, default_value = MODELS[0])]
    model: String,

    #[arg(long, short, default_value = STEPS[1])]
    step: String,

    #[arg(long, short='D', default_value = "1")]
    days: String,

    #[arg(long, short, default_value = DATAS[0])]
    data: String,

    #[arg(long, short='L', default_value = "44,45")]
    lat: String,

    #[arg(long, short, default_value = "5,6")]
    lon: String,

    #[arg(long, short, default_value = ".")]
    outdir: String,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    SelfUpdate,
}

fn error_exit(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    std::process::exit(1);
}

fn parse_coords(coord_str: &str) -> Vec<f64> {
    let coord: Vec<&str> = coord_str.split(',').collect();
    if coord.len() != 2 {
        error_exit("each --lat and --lon must contain exactly two elements separated by a comma");
    }
    let mut result = Vec::with_capacity(2);
    for c in coord {
        match c.trim().parse::<f64>() {
            Ok(nb) => result.push(nb),
            Err(_) => error_exit("Both elements must be valid numbers ex: --lat 6.64,7"),
        }
    }
    result
}
fn init_logger() {
    Builder::new()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            writeln!(
                buf,
                "{} [{}] {}",
                ts,
                record.level(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    init_logger();

    let args = Cli::parse();

    match &args.command {

        Some(Commands::SelfUpdate) => {
            if let Err(e) = github::Update::configure()
                .repo_owner("Ch1nkara")
                .repo_name("GRIB2Sail")
                .bin_name("grib2sail-cli")
                .show_download_progress(true)
                .current_version(env!("CARGO_PKG_VERSION"))
                .build()?
                .update()
            {
                eprintln!("Update failed: {}", e);
                process::exit(1)
            } else {
                println!("Updated successfully!");
            }
        }
        None => {
            println!("non self updater provided");
        }
    }

    let latitudes = parse_coords(&args.lat);
    let longitudes = parse_coords(&args.lon);

    println!("Model: {}", args.model);
    println!("Step: {}", args.step);
    println!("Days: {}", args.days);
    println!("Data: {}", args.data);
    println!("Latitudes: {:?}", latitudes);
    println!("Longitudes: {:?}", longitudes);
    println!("Output Directory: {:?}", args.outdir);
    println!("debug: {}", args.debug);
    //utils::download_grib(model, step, days, data, lat, lon)
    download_grib("dummy test");
    Ok(())
}

