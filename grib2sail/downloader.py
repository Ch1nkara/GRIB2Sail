from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
import requests
from pathlib import Path
from rich.progress import Progress

from grib2sail.logger import logger
from grib2sail.downloader_arom import download_arom
from grib2sail.downloader_gfs import download_gfs
import grib2sail.variables as v

thread_local = threading.local()

def get_session():
  if not hasattr(thread_local, 'session'):
    thread_local.session = requests.Session()
  return thread_local.session

def download_gribs(model, step, days, data, lat, lon):
  if model.startswith('arome'):
    download_arom(model, step, days, data, lat, lon)
  elif model == 'gfs':
    download_gfs(model, step, days, data, lat, lon)
  else:
    logger.error_exit(f"Downloader failed: unexpected model: {model}")

# Optimized resource fetcher with threading and common session
def get_layers(model, urls, header={}):
  # Downloading every layers
  layers = [None] * len(urls)
  with Progress() as progress:
    # Showing a progress bar
    task = progress.add_task('Downloading layers...', total=len(urls))

    # Downloading the layer
    with ThreadPoolExecutor(max_workers=10) as executor:
      futures = [
        executor.submit(fetch, i, url, header, model)
        for i, url in enumerate(urls)
      ]

      for future in as_completed(futures):
        idx, layer = future.result()
        layers[idx] = layer
        progress.advance(task)
  return layers

# Fetch an url and handle errors differently depending on the model
def fetch(idx, url, headers, model):
  try:
    session = get_session()
    r = session.get(url, headers=headers,timeout = 60)
    r.raise_for_status()
    return idx, r.content
  except Exception as e:
    logger.error_exit(f"Download failed: {e}")
    return idx, None

# Output the file once all the layers have been downloaded
def write_file(model, run, step, layers):
  file = Path(f"{model}_{run}_{step}.grib2")
  file.unlink(missing_ok=True)
  with open(file, "wb") as outfile:
    for layer in layers:
      if layer:
        outfile.write(layer)

