# GRIB2Sail

<p align="center">
  <img src="https://raw.githubusercontent.com/Ch1nkara/GRIB2Sail/main/docs/assets/grib2sail_logo.png" alt="GRIB2Sail" width="40%">
</p>
<p align="center">
  <em>Grib files downloader for sailing purposes</em>
</p>
<p align="center">
  <img src="https://img.shields.io/pypi/v/grib2sail.svg">
  <img src="https://img.shields.io/github/actions/workflow/status/Ch1nkara/GRIB2Sail/release.yml">
  <img src="https://img.shields.io/badge/license-GPL%20v3-blue.svg">
</p>

Currently, the supported models are:

- Worldwide models:
  - GFS by NOAA (US):
    - gfs025 (resolution: 0.25° - 22 km)
    - gfs050 (resolution: 0.50° - 45 km)
    - gfs100 (resolution: 1.00° - 90 km)

- Local models:
  - Arome by MeteoFrance (FR):
    - arome (France - resolution: 1 km)
    - arome0025 (France - resolution: 2.5 km)
    - arome-antille (Caribbean area - resolution: 2.5 km)
    - arome-guyane (French Guiana area - resolution: 2.5 km)
    - arome-indien (Mayotte and Réunion Island area - resolution: 2.5 km)
    - arome-ncaledonie (New Caledonia area - resolution: 2.5 km)
    - arome-polynesie (French Polynesia area - resolution: 2.5 km)

## Installation

To install the cli follow the instructions in the [release page](https://github.com/Ch1nkara/GRIB2Sail/releases)
## Meteofrance prerequiste

To download GRIB from meteofrance's models (Aome), you must create a free 
account on meteofrance.fr. The procedure is as follow:
 1. Create an account on [the Météo-France API portal](https://portail-api.meteofrance.fr)
 2. Subscribe to the desired service (Arome)
 3. Go to "My API" then, from your subscribed model: "Generate Token"
 4. Checkout the curl field at the bottom, it looks like :
   ```sh
   curl -k -X POST https://portal-api.meteofrance.fr/token -d "grant_type=client_credentials" -H "Authorization: Basic ABCDEF1234abcdef"
   ```
 5. The string that comes after Basic is your application ID 
   (ABCDEF1234abcdef in this example)
 6. Copy/paste this application ID to GRIB2Sail when prompted (you will only
   be prompted the first time you use GRIB2Sail)

## Usage

To get the GRIB file containing the wind, the wind_gust, the atmospheric 
pressure and the cloud coverage for the area between latitude 11.5N - 12.5N 
and longitude 62.5W - 61.5W with a 1 hour step for 2 days from the arome-antille model 
run:
```sh
grib2sail-cli --model arome-antille --lat 11.5,12.5 --lon -62.5,-61.5 --step 1h --days 2 --components wind,wind-gust,cloud-cover,pressure --outdir .
```

The first time you will be prompted to enter your application ID. It will be 
stored in your keyring for subsequent run.

The downloaded grib file will be present in the working directory named `arome_antille_19700101-00Z_1h.grib2`.

It can now be imported in a navigation software such as OpenCPN

## Iridium-Go

The worldwide models can be downloaded via Iridium-Go by using the flag `--iridium`

## Update

The cli can be updated without going through the installation process by running:
```sh
grib2sail-cli --self-update
```

## Uninstall

Simply delete the file grib2sail-cli

## Roadmap

Main upcoming features:
 - adding more supported models (arpege, ecmwf...)
 - adding more supported variables (rain, sea state)
 - adding a application (android, windows, linux)
