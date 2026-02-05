import typer
import os
import logging

import grib2sail.variables as v
from grib2sail.downloader import download_gribs
from grib2sail.logger import logger

app = typer.Typer(help='Download GRIB2 meteorological data')

# main cli entry point
@app.command()
def main(
  model: str = typer.Option(v.MODELS[0], help='Choose one among: ' + ', '.join(v.MODELS)),
  step: str = typer.Option(v.STEPS[1], help='Choose one among: '  + ', '.join(v.STEPS)),
  days: str = typer.Option('2', help='Forecast duration in days'),
  data: str = typer.Option(v.DATAS[0], help='Choose multiple among: ' + ', '.join(v.DATAS)),
  lat: str = typer.Option(..., help='latitudes max and min ex: -7,-2'),
  lon: str = typer.Option(..., help='longitude max and min ex: -62,-60'),
  outdir: str = typer.Option('.', help='Directory where the grib file will be saved'),
  debug: bool = typer.Option(False, help='Enable debug prints'),
):
  if debug:
    logger.setLevel(logging.DEBUG)
  data = data.split(',')
  lat = parse_coord(lat)
  logger.debug(f"latitude is now: {lat}")
  lon = parse_coord(lon)
  logger.debug(f"longitude is now: {lon}")
  validate_input(model, step, days, data, lat, lon, outdir)
  logger.debug(f"model: {model}, step: {step}, days: {days}, data: {data}")
  logger.info(f"Downloading from {model}: {data}")
  download_gribs(model, step, days, data, lat, lon, outdir)
  logger.info('Done')

## HELPER FUNCTIONS
def parse_coord(coords):
  res = []
  coords = coords.split(',')
  for coord in coords:
    res += [convert_to_nb(coord)]
  return res

def convert_to_nb(nb_str):
  try:
    return int(nb_str)
  except Exception:
    try:
      return float(nb_str)
    except Exception:
      msg = f"failed to convert to int or float: {nb_str}"
      raise typer.BadParameter(msg)

def validate_input(model, step, days, data, lat, lon, outdir):
  # Validate that the model requested is valid
  if model not in v.MODELS:
    logger.error_exit('model must be one of: ' + '|'.join(v.MODELS))
  # Validate that the step requested is valid
  if step not in v.STEPS:
    logger.error_exit('step must be one of: ' + '|'.join(v.STEPS))
  # Validate that the number of days requested is valid
  try:
    int(days)
  except ValueError:
    logger.error_exit('days must be an integer, ex --days 4')
  # Validate that the data requested is valid
  for elmnt in data:
    if elmnt not in v.DATAS:
      msg = 'data must be a combinaison of: '
      logger.error_exit(msg + ','.join(v.DATAS))
  # Validate that the lat and lon requested are valid
  if len(lat) != 2 or len(lon) != 2:
    logger.error_exit('lat and lon must have 2 values each, ex --lat -7,-2')
  for coord in lat:
    if not (-90 <= coord <= 90):
      logger.error_exit('latitude must be between -90 and 90')
  for coord in lon:
    if not (-180 <= coord <= 180):
      logger.error_exit('longitude must be between -180 and 180')
  # Validate that the output directory requested is valid
  if not os.path.isdir(outdir):
    logger.error_exit('outdir must be an existing directory')

if __name__ == '__main__':
  app()

