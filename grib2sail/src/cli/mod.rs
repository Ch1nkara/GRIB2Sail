mod keyring;
mod logger;
mod updater;

use grib2sail as g2s;

use clap::{ArgAction, Parser};
use log::{LevelFilter, debug, error, info};
use std::{fs, path::Path, process};
use tokio::{spawn, sync::mpsc::unbounded_channel};

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
        long, short,
        value_parser = clap::value_parser!(g2s::Component),
        value_delimiter = ',',
        default_value = "wind,wind-gust")
    ]
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

    #[arg(long, short, action = ArgAction::SetTrue)]
    iridium: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    debug: bool,

    #[arg(long, action = ArgAction::SetTrue)]
    reset_keyring: bool,

    #[arg(long, short='u', action = ArgAction::SetTrue)]
    self_update: bool,
}

pub async fn start_cli() {
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

    let secret = match keyring::get_secret(&args.model).await {
        Ok(s) => s,
        Err(e) => error_exit(&e.to_string()),
    };

    let grib = g2s::Grib {
        model: args.model,
        step: args.step,
        days: args.days,
        latitude_min: lat.iter().cloned().fold(f64::INFINITY, f64::min),
        latitude_max: lat.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        longitude_min: lon.iter().cloned().fold(f64::INFINITY, f64::min),
        longitude_max: lon.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        components: args.components,
        iridium: args.iridium,
        content: Vec::new(),
        run: String::new(),
        secret,
    };
    debug!("Grib generated is: {:?}", grib);

    let (tx, mut rx) = unbounded_channel();
    let handle = spawn(async move { g2s::download_grib(grib, tx).await });

    while let Some(event) = rx.recv().await {
        match event {
            g2s::DownloadEvent::Started { total } => {
                if let Err(e) = logger::set_progress_bar(total) {
                    error_exit(&format!("Failed to set progress bar: {}", e));
                }
            }
            g2s::DownloadEvent::FinishedOne => {
                if let Err(e) = logger::increment_progress_bar(1) {
                    error_exit(&format!("Failed to inc progress bar: {}", e));
                }
            }
            g2s::DownloadEvent::FinishedAll => {
                if let Err(e) = logger::clear_progress_bar() {
                    error_exit(&format!("Failed to clear progress bar: {}", e));
                }
            }
        }
    }

    let grib: g2s::Grib;
    match handle.await {
        Ok(handle_result) => match handle_result {
            Ok(grib_res) => grib = grib_res,
            Err(e) => {
                error_exit(&format!("Failed to download the grib: {}", e))
            }
        },
        Err(e) => error_exit(&format!("Failed to spawn subprocess: {}", e)),
    };
    if let Err(e) = logger::clear_progress_bar() {
        error_exit(&format!("Failed to clear progress bar: {}", e));
    }

    let filename = format!("{}_{}_{}.grib2", grib.model, grib.run, grib.step,);
    match fs::write(outdir.join(&filename), grib.content) {
        Ok(_) => {
            info!("Successfully downloaded {}", filename)
        }
        Err(e) => {
            error!("Failed to write the grib file: {}", e)
        }
    }
}

fn error_exit(msg: &str) -> ! {
    error!("{}", msg);
    process::exit(1);
}

fn parse_coords(coord_str: &str) -> Result<Vec<f64>, g2s::GribError> {
    let coord: Vec<&str> = coord_str.split(':').collect();
    if coord.len() != 2 {
        let mut msg = String::from("Each --lat and --lon must contain");
        msg.push_str(" exactly two coordinates separated by a comma.");
        msg.push_str(" Ex: --lat 5.55:6.05");
        return Err(g2s::GribError::InvalidConf(msg));
    }
    let mut result = Vec::with_capacity(2);
    for c in coord {
        match c.trim().replace(',', ".").parse::<f64>() {
            Ok(nb) => result.push(nb),
            Err(_) => {
                let mut msg = String::from("Each --lat and --lon must be");
                msg.push_str(" valid numbers. Ex: --lat 5.5:6.3");
                return Err(g2s::GribError::InvalidConf(msg));
            }
        }
    }
    Ok(result)
}
