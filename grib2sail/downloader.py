from rich.progress import Progress
from pathlib import Path
import requests
import re

from grib2sail.logger import logger
import grib2sail.variables as v
from grib2sail.token import get_arome_token

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
    
  # Download all layers as individual grib files into one output file
  file = Path(f"{model}_{latestRun}_{step}.grib2")
  file.unlink(missing_ok=True)
  with open(file, "ab") as outfile, Progress() as progress:
    # Select forecast prevision time based on user input
    # 3600 means layer is the prevision for 1h after latestRun
    times = list(range(
      int(step[:-1]) * 3600, 
      172800+1, 
      int(step[:-1]) * 3600)
    )
    logger.debug(f"forecast to downloads are {times}")
    # Showing a progress bar
    task = progress.add_task(
      'Downloading layers...', 
      total = len(coverages) * len(times)
    )
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
        url=v.AROM_URLS[f"{model}_cov"]+ paramCovId + paramSubset
        logger.debug(f"Downloading {url}")
        try:
          r = requests.get(
            url, 
            headers = {'Authorization': f"Bearer {token}"},
            timeout = 60)
          r.raise_for_status()
          outfile.write(r.content)
        except requests.exceptions.HTTPError:
          layer = next(
            (k for k, v in v.AROM_DATAS.items() if v == coverage),
            None
          )
          logger.warning(f"Missing layer: {layer} at time: {int(time / 3600)}h")
          logger.debug(
            f"Url used was {url} and status code was {r.status_code}"
          )
        except Exception as e:
          logger.error_exit(f"Download failed: {e}", to_clean=[file])
        layer = next(
          (k for k, v in v.AROM_DATAS.items() if v == coverage), 
          None
        )
        logger.warning(f"Missing layer: {layer} at time: {int(time / 3600)}h")
        progress.update(task, advance=1)
