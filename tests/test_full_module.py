import subprocess
import glob
import os
import pytest
import time as t

@pytest.mark.parametrize("model, step, days, data, lat, lon, outdir", [
   ('gfs', '3h', '5', 'wind,wind_gust,pressure,cloud', '-17,-16', '-150,-149', '.'),
   ('arome', '1h', '1', 'wind,wind_gust,pressure,cloud', '43,44', '5,6', '.'),
   ('arome_polynesie', '12h', '1', 'wind,wind_gust,cloud,pressure', '-17,-16', '-150,-149', '.'),
   ('arome_guyane', '6h', '1', 'wind_gust', '5,6', '-53,-52', '~/'),
   ('arome_antilles', '3h', '1', 'wind_gust', '16.33,17', '-62,-61.33', '.'),
   ('arome_ncaledonie', '1h', '1', 'wind_gust', '-23,-22', '166,167', '..'),
   ('arome_indien', '12h', '2', 'wind_gust', '-21,-20', '55,56', '/tmp/'),
   ('arome0025', '12h', '12', 'wind_gust', '43,44', '5,6', '.'),
   ('gfs', '1h', '33', 'wind_gust', '-17,-16', '-150,-149', '/tmp'),
])
def test_full_module(model, step, days, data, lat, lon, outdir):
    cmd = [
        'python', '-m', 'grib2sail',
        '--model', model,
        '--step', step,
        '--days', days,
        '--data', data,
        '--lat', lat, '--lon', lon,
        '--debug'
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)

    # Check the exit code is 0
    assert result.returncode == 0, f"Non-zero exit code: {result.stdout}"

    # Check the file exists and is non-empty
    pattern = model + '*.grib2'
    files = glob.glob(pattern)
    file_path = files[0] if files else None
    assert file_path is not None, f"No files matching pattern found{result.stdout}"
    assert os.path.getsize(file_path) > 0, f"{file_path} is empty"

    # Clean up after test
    os.remove(file_path)

