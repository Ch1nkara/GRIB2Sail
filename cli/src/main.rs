use clap::Parser;
use chrono::Local;
use env_logger::Builder;
use log::Level;
use std::io::Write;

use rust_core::utils;

#[derive(Parser, Debug)]
#[command(name = "grib2sail_cli")]
#[command(about= "A cli for GRIB2Sail", long_about = None)]
struct Cli {
    #[arg(long, short, default_value = utils::MODELS[0])]
    model: String,

    #[arg(long, short, default_value = utils::STEPS[1])]
    step: String,

    #[arg(long, short='D', default_value = "2")]
    days: String,

    #[arg(long, short, default_value = utils::DATAS[0])]
    data: String,

    #[arg(long, short='L')]
    lat: String,

    #[arg(long, short)]
    lon: String,

    #[arg(long, short, default_value = ".")]
    outdir: String,

    #[arg(long, action = clap::ArgAction::SetTrue)]
    debug: bool,
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

fn main() {
    init_logger();
    let args = Cli::parse();

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
    utils::download_grib("dummy test");
}

