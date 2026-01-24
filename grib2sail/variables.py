MODELS = ['arome_antilles', 'arome001']
STEPS = ['1h', '3h', '6h', '12h']
DATAS = ['wind', 'wind_gust', 'pressure', 'cloud', 'rain']

AROM_DATAS = {
  'wind_u': 'U_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___',
  'wind_v': 'V_COMPONENT_OF_WIND__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___',
  'wind_gust': 'WIND_SPEED_GUST__SPECIFIC_HEIGHT_LEVEL_ABOVE_GROUND___',
  'pressure': 'PRESSURE__MEAN_SEA_LEVEL___',
  'cloud': 'TOTAL_CLOUD_COVER__GROUND_OR_WATER_SURFACE___'
}

AROM_URLS = {
  'token': 'https://portail-api.meteofrance.fr/token',
  f"{MODELS[0]}_cov": 'https://public-api.meteofrance.fr/public/arome/1.0/wcs/MF-NWP-HIGHRES-AROME-OM-0025-ANTIL-WCS/GetCoverage?service=WCS&version=2.0.1&format=application/wmo-grib',
  f"{MODELS[1]}_cov": 'https://public-api.meteofrance.fr/public/arome/1.0/wcs/MF-NWP-HIGHRES-AROME-001-FRANCE-WCS/GetCoverage?service=WCS&version=2.0.1&format=application/wmo-grib',
  f"{MODELS[0]}_capa": 'https://public-api.meteofrance.fr/public/arome/1.0/wcs/MF-NWP-HIGHRES-AROME-OM-0025-ANTIL-WCS/GetCapabilities?service=WCS&version=1.3.0&language=eng',
  f"{MODELS[1]}_capa": 'https://public-api.meteofrance.fr/public/arome/1.0/wcs/MF-NWP-HIGHRES-AROME-001-FRANCE-WCS/GetCapabilities?service=WCS&version=1.3.0&language=eng',
}




