from pathlib import Path
import re
import requests
import time as t

import grib2sail.variables as v
import grib2sail.variables_arom as va
import grib2sail.downloader as d
from grib2sail.logger import logger
from grib2sail.token import get_arome_token

def download_arom(model, step, data, lat, lon):
  token = get_arome_token()
  
  # Coverages list all the individual layers categories to download
  coverages = []
  if v.DATAS[0] in data:
    coverages += [va.AROM_DATAS['wind_u'], va.AROM_DATAS['wind_v']]
  if v.DATAS[1] in data:
    coverages += [va.AROM_DATAS['wind_gust']]
  if v.DATAS[2] in data:
    coverages += [va.AROM_DATAS['pressure']]
  if v.DATAS[3] in data:
    coverages += [va.AROM_DATAS['cloud']]
  
  # Get latest available forecast date from arome /GetCapabilities api endpoint
  logger.info('Finding latest available forecast')
  session = d.get_session()
  try:
    capa = session.get(
      va.AROM_URLS[f"{model}_capa"], 
      headers = {'Authorization': f"Bearer {token}"},
      timeout = 60,
    )
  except Exception as e:
    logger.error_exit(f"Failed to contact METEO FRANCE servers: {e}")
  
  # Parse the GetCapabilities XML response to find the latest available coverage
  lines = [line for line in capa.text.splitlines() if coverages[0] in line]
  if lines:
    # Forecast available dates look like 1970-01-01T00:00:00Z
    # The last line holds the lastest available forecast run
    latestRun = re.search(
      r"\d{4}-\d{2}-\d{2}T\d{2}\.\d{2}\.\d{2}Z",
      lines[-1]
    )
    if latestRun:
      latestRun = latestRun.group()
    else:
      msg = "Error fetching AROM capabilities, couldn't find latest date"
      logger.error_exit(msg)
  else:
    msg = "Error fetching AROM capabilities, couldn't find latest run"
    logger.error_exit(msg)

  # Select forecast prevision time based on user input
  # 3600 means layer is the prevision for 1h after latestRun
  times = list(range(
    int(step[:-1]) * 3600,
    172800+1,
    int(step[:-1]) * 3600)
  )
  logger.debug(f"Forecast to download are {times}")

  # Generating the urls to retreive requested layers
  header = {'Authorization': f"Bearer {token}"}
  urls = []
  for coverage in coverages:
    for time in times:
      paramCovId = f"&coverageid={coverage}{latestRun}"
      subTime = f"&subset=time({time})"
      subLat = f"&subset=lat({lat[0]},{lat[1]})"
      subLon = f"&subset=long({lon[0]},{lon[1]})"
      if 'SPECIFIC_HEIGHT' in coverage:
        subHeight = '&subset=height(10)'
      else:
        subHeight = ''
      paramSubset = subTime + subLat + subLon + subHeight
      urls.append(va.AROM_URLS[f"{model}_cov"]+ paramCovId + paramSubset)

  # Downloading the layers
  layers = []
  if len(urls) < 100:
    layers = d.get_layers(model, urls, header)
  else:
    msg = f"The requested grib has {len(urls)} layers, but MeteoFrance"
    msg += ' servers limit requests to 100 per minute. This program will'
    msg += ' sleep 1 minute every 100 layer util the complete grib file'
    msg += ' is downloaded. You might want to consider reducing the number'
    msg += ' of layers by increasing the step or reducing the number of'
    msg += ' data'
    logger.warning(msg)
    for i in range(0, len(urls), 100):
      layers.extend(d.get_layers(model, urls[i:i+100], header))
      if i+100 < len(urls):
        logger.info('Sleeping 1 minute...')
        t.sleep(60)
  
  # Output the file once all the layers have been downloaded
  file = Path(f"{model}_{latestRun}_{step}.grib2")
  file.unlink(missing_ok=True)
  with open(file, "wb") as outfile:
    for layer in layers:
      if layer:
        outfile.write(layer)

def handle_fetch_error_arom(e):
  if isinstance(e, requests.exceptions.HTTPError):
    url = e.response.url
    layer = re.search(r"coverageid=(.*?)__", url).group(1)
    time = int(re.search(r"subset=time\(([^()]*)", url).group(1)) / 3600
    logger.warning(f"Missing layer: {layer} at time: {int(time)}h")
    logger.debug(f"Error was {e}")
  else:
    logger.error_exit(f"Download failed: {e}")
