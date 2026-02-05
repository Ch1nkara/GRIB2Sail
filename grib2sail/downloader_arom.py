import re
import requests
import time as t

import grib2sail.variables as v
import grib2sail.variables_arom as va
import grib2sail.downloader as d
from grib2sail.logger import logger
from grib2sail.token import get_arome_token

def download_arom(model, step, days, data, lat, lon):
  token = get_arome_token()
  header = {'Authorization': f"Bearer {token}"}

  # Coverages list all the individual layers categories to download
  coverages = []
  for param in data:
    if param == v.DATAS[0]:
      coverages += [va.AROM_DATAS['wind_u'], va.AROM_DATAS['wind_v']]
    else:
      coverages += [va.AROM_DATAS[param]]

  # Get latest available forecast date from arome /GetCapabilities api endpoint
  logger.info('Finding latest available forecast')
  session = d.get_session()
  try:
    capa = session.get(
      va.AROM_URLS[f"{model}_capa"], 
      headers = header,
      timeout = 30,
    )
  except Exception as e:
    logger.error_exit(f"Failed to contact METEO FRANCE servers: {e}")
  
  # Parse the GetCapabilities XML response to find the latest available coverage
  lines = [line for line in capa.text.splitlines() if coverages[0] in line]
  # Forecast available dates look like 1970-01-01T00:00:00Z
  # The last 2 lines hold the 2 lastest available forecast run
  last2run = []
  if len(lines) >= 2:
    for line in lines[-2:][::-1]:
      match = re.search(r"\d{4}-\d{2}-\d{2}T\d{2}\.\d{2}\.\d{2}Z", line)
      if match:
        last2run.append(match.group())
      else:
        msg = "Error fetching AROM capabilities, couldn't parse latest run"
        logger.error_exit(msg)
  else:
    msg = "Error fetching AROM capabilities, couldn't find latest run"
    logger.error_exit(msg)
  logger.debug(f"latest 2 runs are {last2run[0]} and {last2run[1]}")

  # Select forecast prevision time based on user input
  # 3600 means layer is the prevision for 1h after latestRun
  nbDay = 1 if days == "1" else 2
  times = list(range(
    int(step[:-1]) * 3600,
    nbDay * 24 * 60 * 60 + 1,
    int(step[:-1]) * 3600)
  )
  logger.debug(f"Forecast to download are {times}")

  # Generating the urls to retreive requested layers
  latestRun = last2run[0]
  urls = generate_arom_layers_urls(model, coverages, latestRun, times, lat, lon)

  # If the last run does not have all the required layers yet,
  # fallback to the previous run
  try:
     lastLayer = session.get(
      urls[-1],
      headers = header,
      timeout = 30,
    )
  except Exception as e:
    logger.warning('The latest run does not have all the layers yet, using the one before')
    latestRun = last2run[1]
    urls = generate_arom_layers_urls(model, coverages, latestRun, times, lat, lon)

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

  # Format output file name 
  run = latestRun.replace('-', '')
  run = re.sub(r'T(00|06|12|18)\.00\.00Z', r'-\1z', run)

  return layers, run

def generate_arom_layers_urls(model, coverages, latestRun, times, lat, lon):
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
  return urls

