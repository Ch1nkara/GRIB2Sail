from pathlib import Path
import re
from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
import requests
from rich.progress import Progress

from grib2sail.logger import logger
import grib2sail.variables as v
from grib2sail.token import get_arome_token

thread_local = threading.local()

def get_session():
  if not hasattr(thread_local, 'session'):
    thread_local.session = requests.Session()
  return thread_local.session

def download_gribs(m, s, d, lat, lon):
  if m.startswith('arome'):
    download_arome(m, s, d, lat, lon)
  else:
    logger.error_exit(f"Downloader failed: unexpected model: {m}")

def download_arome(model, step, data, lat, lon):
  token = get_arome_token()
  
  # Coverages list all the individual layers categories to download
  coverages = []
  if v.DATAS[0] in data:
    coverages += [v.AROM_DATAS['wind_u'], v.AROM_DATAS['wind_v']]
  if v.DATAS[1] in data:
    coverages += [v.AROM_DATAS['wind_gust']]
  if v.DATAS[2] in data:
    coverages += [v.AROM_DATAS['pressure']]
  if v.DATAS[3] in data:
    coverages += [v.AROM_DATAS['cloud']]
  
  # Get latest available forecast date from arome /GetCapabilities api endpoint
  logger.info('Finding latest available forecast')
  try:
    capa = requests.get(
      v.AROM_URLS[f"{model}_capa"], 
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
  logger.debug(f"Forecast to downloads are {times}")

  # Generating the urls to retreive requested layers
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
      urls.append(v.AROM_URLS[f"{model}_cov"]+ paramCovId + paramSubset)

  # Downloading every layers
  layers = [None] * len(urls)
  with Progress() as progress:
    # Showing a progress bar
    task = progress.add_task('Downloading layers...', total=len(urls))

    # Downloading the layer
    header = {'Authorization': f"Bearer {token}"}
    with ThreadPoolExecutor(max_workers=10) as executor:
      futures = [
        executor.submit(fetch_arom, i, url, header)
        for i, url in enumerate(urls)
      ]

      for future in as_completed(futures):
        idx, layer = future.result()
        layers[idx] = layer
        progress.advance(task)

  # Output the file once all the layers have been downloaded
  file = Path(f"{model}_{latestRun}_{step}.grib2")
  file.unlink(missing_ok=True)
  with open(file, "wb") as outfile:
    for layer in layers:
      outfile.write(layer)

def fetch(idx, url, headers):
  session = get_session()
  r = session.get(url, headers=headers,timeout = 60)
  r.raise_for_status()
  return idx, r.content

def fetch_arom(idx, url, headers):
  try:
    return fetch(idx, url, headers)
  except requests.exceptions.HTTPError as e:
    layer = re.search(r"coverageid=(.*?)__", url).group(1)
    time = int(re.search(r"subset=time\(([^()]*)", url).group(1)) / 3600

    logger.warning(f"Missing layer: {layer} at time: {int(time)}h")
    logger.debug(f"Error was {e}")
    return idx, None
  except Exception as e:
    logger.error_exit(f"Download failed: {e}")
