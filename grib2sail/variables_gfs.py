PROD_URL = 'https://nomads.ncep.noaa.gov/pub/data/nccf/com/gfs/prod'
API_URL = 'https://nomads.ncep.noaa.gov/cgi-bin/filter_gfs_0p25.pl'

GFS_DATAS = {
  'wind_u': '&var_UGRD=on&lev_10_m_above_ground=on',
  'wind_v': '&var_VGRD=on',
  'wind_gust': '&var_GUST=on&lev_surface=on',
  'pressure': '&var_PRMSL=on&lev_mean_sea_level=on',
  'cloud': '&var_TCDC=on&lev_entire_atmosphere=on'
}

