use clap::{Parser, ArgAction};
use indicatif::ProgressBar;
use log::{error, debug, info};
use std::{fs, process, path::Path};

use grib2sail as g2s;

mod logger;
mod updater;
mod keyring;

#[derive(Parser, Debug)]
#[command(name = "grib2sail-cli")]
#[command(about = "A cli for GRIB2Sail", long_about = None, version)]
struct Cli {
    #[arg(long, short, value_enum, default_value_t = g2s::Model::Arome)]
    model: g2s::Model,

    #[arg(long, short, value_enum, default_value_t = g2s::Step::H3)]
    step: g2s::Step,

    #[arg(long, short, default_value_t = 1)]
    days: u32,

    #[arg(
        long, short,
        value_parser = clap::value_parser!(g2s::Component),
        value_delimiter = ',',
        default_value = "wind")
    ]
    components: Vec<g2s::Component>,

    #[arg(long, short='L', allow_hyphen_values = true, default_value = "44,45")]
    lat: String,

    #[arg(long, short, allow_hyphen_values = true, default_value = "5,6")]
    lon: String,

    #[arg(long, short, default_value = ".")]
    outdir: String,

    #[arg(long, action = ArgAction::SetTrue)]
    debug: bool,

    #[arg(long, short='u', action = ArgAction::SetTrue)]
    self_update: bool,
}

pub async fn start_cli(){
    let args = Cli::parse();

    if args.debug {
        logger::init(4);
    } else {
        logger::init(3);
    }

    if args.self_update {
        updater::self_update();
        return
    }

    let outdir = Path::new(&args.outdir);
    if ! outdir.is_dir() {
        error_exit("--outdir must be an existing directory")
    }

    let latitudes = parse_coords(&args.lat);
    let longitudes = parse_coords(&args.lon);

    let mut grib = g2s::Grib {
        model: args.model,
        step: args.step,
        days: args.days,
        latitude_min: latitudes.iter().cloned().fold(f64::INFINITY, f64::min),
        latitude_max: latitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        longitude_min: longitudes.iter().cloned().fold(f64::INFINITY, f64::min),
        longitude_max: longitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        components: args.components,
        content: Vec::new(),
        run: String::new(),
        secret: String::new(),
    };
    debug!("Grib generated is \n {:?}", grib);

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let handle = tokio::spawn(async move {
        g2s::download_grib(grib, tx).await
    });

    let pb = ProgressBar::new(100);
    while let Some(event) = rx.recv().await {
        match event {
            g2s::DownloadEvent::Started {total} => pb.set_length(total as u64),
            g2s::DownloadEvent::FinishedOne => pb.inc(1),
            g2s::DownloadEvent::FinishedAll => pb.finish(),
        }
    }

    // TODO remove unwrap double
    let grib = handle.await.unwrap().unwrap();
    debug!("grib is {:?}", grib);

    let filename = format!(
        "{}_{}_{}.grib2",
        grib.model.to_string(),
        grib.run,
        grib.step,
    );
    match fs::write(outdir.join(filename), grib.content) {
        Ok(_) => {info!("Done")},
        Err(e) => {error!("Failed to write the grib file: {}", e)},
    }
}

fn error_exit(msg: &str) -> ! {
    error!("{}", msg);
    process::exit(1);
}

fn parse_coords(coord_str: &str) -> Vec<f64> {
    let coord: Vec<&str> = coord_str.split(',').collect();
    if coord.len() != 2 {
        let mut msg = String::from("Each --lat and --lon must contain exactly two coordinates");
        msg.push_str(" separated by a comma. Ex: --lat 5.55,6.05");
        error_exit(&msg);
    }
    let mut result = Vec::with_capacity(2);
    for c in coord {
        match c.trim().parse::<f64>() {
            Ok(nb) => result.push(nb),
            Err(_) => error_exit("Each --lat and --lon must be valid numbers. Ex: --lat 5.5,6"),
        }
    }
    result
}

