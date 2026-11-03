mod keyring;
mod logger;
mod updater;

use grib2sail as g2s;

use anyhow::Result;
use clap::{ArgAction, Parser};
use log::{LevelFilter, debug, error, info};
use std::{fs, path::Path, process};

#[derive(Parser, Debug)]
#[command(name = "grib2sail-cli")]
#[command(about = "A cli for GRIB2Sail", long_about = None, version)]
struct Cli {
    #[arg(long, short, value_enum, default_value_t = g2s::Model::Gfs025)]
    model: g2s::Model,

    #[arg(long, short, value_enum, default_value_t = g2s::Step::H3)]
    step: g2s::Step,

    #[arg(long, short, default_value_t = 1)]
    days: u32,

    #[arg(
        long,
        short,
        value_enum,
        value_delimiter = ',',
        default_value = "wind,wind-gust"
    )]
    components: Vec<g2s::Component>,

    #[arg(
        long,
        short = 'L',
        allow_hyphen_values = true,
        default_value = "42:43"
    )]
    lat: String,

    #[arg(long, short, allow_hyphen_values = true, default_value = "5:6")]
    lon: String,

    #[arg(long, short, default_value = ".")]
    outdir: String,

    #[arg(long, action = ArgAction::SetTrue)]
    debug: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    reset_keyring: bool,

    #[arg(long, short='u', action = ArgAction::SetTrue)]
    self_update: bool,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let res = if args.debug {
        logger::init(LevelFilter::Debug)
    } else {
        logger::init(LevelFilter::Info)
    };
    if let Err(e) = res {
        error_exit(&format!("Failed to init logger: {}", e));
    }

    if args.self_update {
        match updater::self_update() {
            Ok(_) => return,
            Err(e) => error_exit(&format!("Failed to update the cli: {}", e)),
        }
    }

    if args.reset_keyring {
        match keyring::delete_secrets() {
            Ok(_) => return,
            Err(e) => error_exit(&format!("Failed to reset keyring: {}", e)),
        }
    }

    let outdir = Path::new(&args.outdir);
    if !outdir.is_dir() {
        error_exit("--outdir must be an existing directory")
    }

    let lat = match parse_coords(&args.lat) {
        Ok(l) => l,
        Err(e) => error_exit(&format!("Failed to parse latitudes: {}", e)),
    };
    let lon = match parse_coords(&args.lon) {
        Ok(l) => l,
        Err(e) => error_exit(&format!("Failed to parse longitudes: {}", e)),
    };

    let auth = match keyring::get_secret(&args.model).await {
        Ok(s) => s,
        Err(e) => error_exit(&e.to_string()),
    };

    let options = g2s::DownloadOptions::new()
        .with_auth(auth)
        .with_on_started(|total| {
            logger::set_progress_bar(total).map_err(|e| e.to_string())
        })
        .with_on_finished_one(|inc| {
            logger::increment_progress_bar(inc as u64)
                .map_err(|e| e.to_string())
        })
        .with_on_finished_all(|_| {
            logger::clear_progress_bar().map_err(|e| e.to_string())
        });

    let grib = g2s::Grib {
        model: args.model.clone(),
        step: args.step,
        days: args.days,
        latitude_min: lat.iter().cloned().fold(f64::INFINITY, f64::min),
        latitude_max: lat.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        longitude_min: lon.iter().cloned().fold(f64::INFINITY, f64::min),
        longitude_max: lon.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        components: args.components,
    };
    debug!("Grib generated is: {:?}", grib);

    let (grib_content, run) = match g2s::download_grib(grib, options).await {
        Ok(res) => res,
        Err(e) => error_exit(&e),
    };
    let filename = format!("{}_{}_{}.grib2", args.model, run, args.step,);
    match fs::write(outdir.join(&filename), grib_content) {
        Ok(_) => info!("Successfully downloaded {}", filename),
        Err(e) => error!("Failed to write the grib file: {}", e),
    }
}

fn error_exit(msg: &str) -> ! {
    error!("{}", msg);
    process::exit(1);
}

fn parse_coords(coord_str: &str) -> Result<Vec<f64>> {
    let coord: Vec<&str> = coord_str.split(':').collect();
    if coord.len() != 2 {
        let mut msg = String::from("Each --lat and --lon must contain");
        msg.push_str(" exactly two coordinates separated by a comma.");
        msg.push_str(" Ex: --lat 5.55:6.05");
        return Err(anyhow::anyhow!(msg));
    }
    let mut result = Vec::with_capacity(2);
    for c in coord {
        match c.trim().replace(',', ".").parse::<f64>() {
            Ok(nb) => result.push(nb),
            Err(_) => {
                let mut msg = String::from("Each --lat and --lon must be");
                msg.push_str(" valid numbers. Ex: --lat 5.5:6.3");
                return Err(anyhow::anyhow!(msg));
            }
        }
    }
    Ok(result)
}
