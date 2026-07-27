# GRIB2Sail

<p align="center">
  <img src="https://raw.githubusercontent.com/Ch1nkara/GRIB2Sail/main/docs/assets/grib2sail_logo.png"
    alt="GRIB2Sail" width="40%">
</p>
<p align="center">
  <em>Grib files downloader for sailing purposes</em>
</p>
<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/Ch1nkara/GRIB2Sail/release.yml"
    alt=latest-release>
  <img src="https://img.shields.io/badge/license-GPL%20v3-blue.svg"
    alt=licence>
</p>

Currently, the supported models are:

- Worldwide models:
  - GFS by NOAA (US):
    - gfs025 (resolution: 0.25° - 22 km)
    - gfs050 (resolution: 0.50° - 45 km)
    - gfs100 (resolution: 1.00° - 90 km)
  - IFS by ECMWF (Europe):
    - ecmwf (resolution: 0.25° - 22 km)
  - Arpege by MeteoFrance (FR):
    - Arpege025 (resolution: 0.25° - 22 km)
    - Arpege100 (resolution: 1.00° - 90 km)

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

## Some prerequistes

### Meteofrance models (Arome, Arpege)

To download GRIB from meteofrance's models (Arome, Arpege), you must create a free
account on meteofrance.fr. The procedure is as follow:

1. Create an account on [the Météo-France API portal](https://portail-api.meteofrance.fr)
2. Subscribe to the desired service (Arome, Arpege)
3. Go to "My API" then, from your subscribed model: "Generate Token"
4. Checkout the curl field at the bottom, it looks like :

  ```sh
  curl -k -X POST https://portal-api.meteofrance.fr/token \
    -d "grant_type=client_credentials" \
    -H "Authorization: Basic ABCDEF1234abcdef"
  ```

1. The string that comes after Basic is your API subscription
  (ABCDEF1234abcdef in this example)
2. Copy/paste this subscription to GRIB2Sail when prompted (you will only
  be prompted the first time you use a meteofrance model with GRIB2Sail)

### ECMWF Model

ECMWF model can only be downloaded for the entire earth surface, not just a
specific area. This program will try to call `cdo` binary (Climate Data
Operator, a tool for manipulating grib files) in order to reshape them.
If cdo in not installed on the machine, the full grib file will be provided.

Please note that the full earth grib file can be unreadable by some third
party softwares such as OpenCPN.

To install `cdo` on a Ubuntu machine, use the command `sudo apt install cdo`.

## Usage

To get the GRIB file containing the wind, the wind_gust, the atmospheric
pressure and the cloud coverage for the area between latitude 11.5N - 12.5N
and longitude 62.5W - 61.5W with a 1 hour step for 2 days from the arome-antille
model run:

```sh
grib2sail-cli \
  --model arome-antille \
  --lat 11.5:12.5 \
  --lon -62.5:-61.5 \
  --step 1h --days 2 \
  --components wind,wind-gust,cloud-cover,pressure \
  --outdir .
```

The first time you will be prompted to enter your application ID. It will be
stored in your keyring for subsequent run.

The downloaded grib file will be present in the working directory named `arome_antille_19700101-00Z_1h.grib2`.

It can now be imported in a navigation software such as OpenCPN

## Iridium-Go

NOAA models (gfs025, gfs050, gfs100) can be downloaded via Iridium-Go by
using the flag `--iridium`

## Update

The cli can be updated without going through the installation process by running:

```sh
grib2sail-cli --self-update
```

## Dev

- Create the devcontainer with
`podman build -f .devcontainer/Containerfile -t rust-grib2sail .`
- Run it with:

  ```sh
  podman run -dit --name rust-grib2sail-container \
    -v "$(pwd)":/workspaces/project:Z \
    -v ~/.gitconfig:/root/.gitconfig:ro \
    -v ~/.bashrc:/root/.bashrc:ro \
    -v $SSH_AUTH_SOCK:/ssh-agent \
    -e SSH_AUTH_SOCK=/ssh-agent \
    -w /workspaces/project \
    rust-grib2sail bash
  ```

- Enter it with `podman exec -it rust-grib2sail-container bash`
- Once inside open IDE with `nvim .` and compile from a terminal within with
`cargo build`

## Uninstall

Simply delete the file grib2sail-cli

## Roadmap

Main upcoming features:

- adding more supported models (ICON...)
- adding more supported variables (rain, sea state)
- adding a application (android, windows, linux)
