use crate::core::GribError;
use crate::ecmwf::unpack_crop::{GribParams, get_sect_len, set_4bytes};

pub fn sect_3(
    grib: &[u8],
    idx: usize,
    g_p: &mut GribParams,
) -> Result<Vec<u8>, GribError> {
    let sect_len = get_sect_len(grib, idx);
    let mut res: Vec<u8> = grib[idx..idx + sect_len].to_vec();

    // Verification that the rest of sector is as expected
    if sect_len < 72 {
        return Err(format!("Sect3 too short: {}", sect_len).into());
    }
    if grib[idx + 12] != 0x00 || grib[idx + 13] != 0x00 {
        return Err(format!(
            "Unexpected grid definition 0x{:02X}{:02X}, not lat-lon (0x0000)",
            grib[idx + 12],
            grib[idx + 13]
        )
        .into());
    }
    if get_4bytes(grib, idx + 63) != 0x0003d090
        || get_4bytes(grib, idx + 67) != 0x0003d090
    {
        return Err("Only grib with resolution of 0.25° are supported".into());
    }
    if grib[idx + 71] != 0x00 && grib[idx + 71] != 0x40 {
        return Err(
            format!("Unexpected flag N-S, E-W: {}", grib[idx + 71]).into()
        );
    }

    // Parse the parameters needed to crop the data
    g_p.lat0 = get_4bytes_lat(grib, idx + 46);
    g_p.lon0 = get_4bytes(grib, idx + 50) as f64 / 1_000_000.0;
    if g_p.lon0 >= 180.0 {
        g_p.lon0 -= 360.0
    }
    g_p.lat_end = get_4bytes_lat(grib, idx + 55);
    g_p.lon_end = get_4bytes(grib, idx + 59) as f64 / 1_000_000.0;
    if g_p.lon_end > 180.0 {
        g_p.lon_end -= 360.0
    }

    // Update the parameters to match the cropped data
    let ni =
        ((g_p.longitude_max - g_p.longitude_min) * 4.0).round() as usize + 1;
    let nj = ((g_p.latitude_max - g_p.latitude_min) * 4.0).round() as usize + 1;
    let (lat_ini, lat_end) = if grib[idx + 71] == 0x00 {
        (g_p.latitude_max, g_p.latitude_min)
    } else {
        (g_p.latitude_min, g_p.latitude_max)
    };
    set_4bytes(&mut res, 6, ni * nj);
    set_4bytes(&mut res, 30, ni);
    set_4bytes(&mut res, 34, nj);
    set_position(&mut res, 46, lat_ini, false);
    set_position(&mut res, 50, g_p.longitude_min, true);
    set_position(&mut res, 55, lat_end, false);
    set_position(&mut res, 59, g_p.longitude_max, true);

    Ok(res)
}

fn get_4bytes(grib: &[u8], idx: usize) -> u32 {
    let value = [grib[idx], grib[idx + 1], grib[idx + 2], grib[idx + 3]];
    u32::from_be_bytes(value)
}

fn get_4bytes_lat(grib: &[u8], idx: usize) -> f64 {
    let raw = get_4bytes(grib, idx);
    let value = if (raw >> 31) & 1 == 1 {
        -(raw as i32 & 0x7FFFFFFF)
    } else {
        raw as i32
    };
    value as f64 / 1_000_000.0
}

fn set_position(grib: &mut [u8], idx: usize, value: f64, is_lon: bool) {
    let mut scaled = (value * 1_000_000.0).round() as i32;
    if value < 0.0 && is_lon {
        scaled += 360_000_000;
    } else if value < 0.0 {
        scaled = (-scaled) | 0x80000000u32 as i32;
    }
    let bytes = scaled.to_be_bytes();
    grib[idx..idx + 4].copy_from_slice(&bytes);
}
