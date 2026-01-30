from concurrent.futures import ThreadPoolExecutor, as_completed
import threading
import requests
from rich.progress import Progress

from grib2sail.logger import logger
from grib2sail.downloader_arom import handle_fetch_error_arom, download_arom
import grib2sail.variables as v

thread_local = threading.local()

def get_session():
  if not hasattr(thread_local, 'session'):
    thread_local.session = requests.Session()
  return thread_local.session

def download_gribs(m, s, d, lat, lon):
  if m.startswith('arome'):
    download_arom(m, s, d, lat, lon)
  else:
    logger.error_exit(f"Downloader failed: unexpected model: {m}")

def get_layers(model, urls, header):
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

def fetch(idx, url, headers, model):
  try:
    session = get_session()
    r = session.get(url, headers=headers,timeout = 60)
    r.raise_for_status()
    return idx, r.content
  except Exception as e:
    if model in v.MODELS[:2]:
      handle_fetch_error_arom(e)
    else:
      logger.error_exit(f"Download failed: {e}")
    return idx, None
