# GRIB2Sail

![PyPI version](https://img.shields.io/pypi/v/grib2sail.svg)
![CI](https://img.shields.io/github/actions/workflow/status/Ch1nkara/GRIB2Sail/release.yml)
![License](https://img.shields.io/badge/license-GPL%20v3-blue.svg)

Grib files downloader for sailing purposes.

Currently the supported models are:
 - AROME
 - AROME ANTILLE

## Installation

To install GRIB2Sail simply run:
```bash
pip install grib2sail
```

To download GRIB from meteofrance's models (Aome), you must create a free 
account on meteofrance.fr. The procedure is as follow:
 1. Create an account on [the Météo-France API portal](https://portail-api.meteofrance.fr)
 2. Subscribe to the desired service (Arome)
 3. Go to "My API" then, from your subscribed model: "Generate Token"
 4. Checkout the curl field at the bottom, it looks like :
   ```bash
   curl -k -X POST https://portal-api.meteofrance.fr/token -d "grant_type=client_credentials" -H "Authorization: Basic ABCDEF1234abcdef"
   ```
 5. The string that comes after Basic is your application ID 
   (ABCDEF1234abcdef in this example)
 6. Copy/paste this application ID to GRIB2Sail when prompted (you will only
   be prompted the first time you use GRIB2Sail)

## Usage

To get the GRIB file contianing the wind, the wind_gust, the atmospheric 
pressure and the cloud coverage for the area between latitude 11.5N - 12.5N 
and longitude 62.5W - 61.5W with a 3 hour step from the AROME ANTILLE model 
run:
```bash
python -m grib2sail --lat 11.5,12.5 --lon -62.5,-61.5 --step 3h --data wind,wind_gust,cloud,pressure
```

The first time you will be prompted to enter your application ID. It will be 
stored in your keyring for subsequent run.

The downloaded grib file will be present in the working directory named `arome_antille_1970-01-01T00.00.00Z_3h.grib2`.

It can now be imported in a navigation software such as OpenCPN

## Roadmap

This is still the early stage of the development the main upcoming features 
are:
 - adding more supported models (arpege, gfs, ecmwf...)
 - adding more supported variables (rain, sea state)
 - adding a GUI
