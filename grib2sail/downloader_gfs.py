from datetime import datetime, timedelta

import grib2sail.variables as v
import grib2sail.variables_gfs as vg 
import grib2sail.downloader as d
from grib2sail.logger import logger

def download_gfs(model, step, days, data, lat, lon):
  session = d.get_session()
  # Get latest available forecast date and run by trying from most recent
  logger.info('Finding latest available forecast')
  date, run = find_latest_forecast(session)
  logger.debug(f"Latest forecast is {date}, {run}z")

  # Coverages list all the individual layers categories to download
  coverages = []
  for param in data:
    if param == v.DATAS[0]:
      coverages += [vg.GFS_DATAS['wind_u'], vg.GFS_DATAS['wind_v']]
    else:
      coverages += [vg.GFS_DATAS[param]]

  urls = []
  if int(days) > 16:
    logger.warning(f"Requesting {days} days, max is 16")
    days = "16"
  # Forecast is available on an hourly basis until day 5 then on a 3h basis
  if step == '1h' and days > 5 :
    logger.warning('Only the first 5 days can have a step of 1h, the rest will have a 3h step')
    hours = list(range(0, 120, 1)) + list(range(120, 385, 3))
  else:
      hours = list(range(0, 24 * int(days) + 1, int(step[:-1])))
  for hour in hours:
    url = vg.API_URL 
    url += f"?dir=%2Fgfs.{date}%2F{run}%2Fatmos&file=gfs.t{run}z.pgrb2.0p25.f{hour:03d}"
    url += ''.join(coverages) + f"&subregion="
    url += f"&leftlon={lon[0]}&rightlon={lon[1]}" + f"&bottomlat={lat[0]}&toplat={lat[1]}"
    urls.append(url)
  logger.debug(f"First url to download is {urls[0]}")
  layers = d.get_layers(model, urls)

  # Write the grib file as the concatenation of the layers
  d.write_file(model, f"{date}-{run}z", step, layers)

def find_latest_forecast(session):
  today = datetime.today()
  dates = [
    (today + timedelta(days=1)).strftime("%Y%m%d"), 
    today.strftime("%Y%m%d"), 
    (today - timedelta(days=1)).strftime("%Y%m%d")
  ]
  runs = ['18', '12', '06', '00']
  for date in dates:
    for run in runs:
      url = (
        f"{vg.PROD_URL}/gfs.{date}/{run}/atmos/"
        f"gfs.t{run}z.pgrb2.0p25.f000"
      )
      try:
        r = session.head(url, timeout=10)
        if r.status_code == 200:
          return date, run 
        else:
          logger.debug(f"Unavailable forecast {url}, status is: {r.status_code}")
      except Exception as e:
        logger.error_exit(f"Download failed: {e}")
  logger.error_exit("Couldn't find the latest available forecat")

