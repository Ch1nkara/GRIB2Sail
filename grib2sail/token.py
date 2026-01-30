import os
import keyring
import getpass

import grib2sail.downloader as d
from grib2sail import variables_arom as va
from grib2sail.logger import logger

def get_arome_token():
  logger.info('Authenticating to MeteoFrance')
  appId = get_arome_appid()
  session = d.get_session()
  try:
    response = session.post(
      va.AROM_URLS['token'], 
      data = { 'grant_type': 'client_credentials' }, 
      headers = { 'Authorization': f"Basic {appId}" },
      timeout = 60,
    )
    response.raise_for_status()
  except Exception as e:
    logger.error_exit(f"Failed to get token from METEO FRANCE servers: {e}")
  return response.json()["access_token"]
    

def get_arome_appid():
  # Get appId from env variables
  if appId := os.getenv("GRIB2SAIL_AROME_APPID"):
    return appId
  # Get appId from keyring
  try:
    appId = keyring.get_password('grib2sail', 'arome_appid')
  except Exception as e:
    msg = 'No password storing solution available, install one or use the '
    msg += '"GRIB2SAIL_AROME_APPID" environement variable'
    logger.error_exit(f"{msg}\n{e}")
  if appId is None:
    msg = 'For the first use only, you must create and provide a free '
    msg += 'application ID from meteofrance.fr, it wil be saved locally. '
    msg += 'See documentation for exact procedure'
    logger.info(msg) 
    appId = getpass.getpass('Enter AROME appId: ')
    keyring.set_password('grib2sail', 'arome_appid', appId)
  return appId
