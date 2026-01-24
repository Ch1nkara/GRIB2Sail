import typer
import logging

import grib2sail.variables as v
from grib2sail.downloader import download_gribs
from grib2sail.logger import logger

app = typer.Typer(help='Download GRIB2 meteorological data')

# main cli entry point
@app.command()
def main(
  model: str = typer.Option(v.MODELS[0], help='|'.join(v.MODELS)),
  step: str = typer.Option(v.STEPS[1], help='|'.join(v.STEPS)),
  data: str = typer.Option(v.DATAS[0], help=','.join(v.DATAS)),
  lat: str = typer.Option(..., help='latitudes max and min ex: -7,-2'),
  lon: str = typer.Option(..., help='longitude max and min ex: -62,-60'),
  debug: bool = typer.Option(False, help='Enable debug prints'),
):
  if debug:
    logger.setLevel(logging.DEBUG)
  data = data.split(',')
  lat = parse_coord(lat)
  logger.debug(f"latitude is now: {lat}")
  lon = parse_coord(lon)
  logger.debug(f"longitude is now: {lon}")
  validate_input(model, step, data, lat, lon)
  
  logger.debug(f"model: {model}, step: {step}, data: {data}")
  logger.info(f"Downloading from {model}: {data}")
  download_gribs(model, step, data, lat, lon)
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

def validate_input(m, s, d, lat, lon):
  if m not in v.MODELS:
    logger.error_exit('model must be one of: ' + '|'.join(v.MODELS))
  if s not in v.STEPS:
    logger.error_exit('step must be one of: ' + '|'.join(v.STEPS))
  for elmnt in d:
    if elmnt not in v.DATAS:
      msg = 'data must be a combinaison of: '
      logger.error_exit(msg + ','.join(v.DATAS))
  if len(lat) != 2 or len(lon) != 2:
    logger.error_exit('lat and lon must have 2 values each, ex --lat -7,-2')
  for coord in lat:
    if not (-90 <= coord <= 90):
      logger.error_exit('latitude must be between -90 and 90')
  for coord in lon:
    if not (-180 <= coord <= 180):
      logger.error_exit('longitude must be between -180 and 180')
  
if __name__ == '__main__':
  app()

  
