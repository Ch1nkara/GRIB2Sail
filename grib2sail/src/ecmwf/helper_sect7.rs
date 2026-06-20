use crate::core::GribError;
use crate::ecmwf::unpack_crop::{GribParams, get_sect_len, set_4bytes};

use log::{debug, error};
use rust_aec::{AecParams, decode, flags_from_grib2_ccsds_flags};

pub fn sect_7(
    grib: &[u8],
    mut idx: usize,
    g_p: &GribParams,
) -> Result<Vec<u8>, GribError> {
    let sect_len = get_sect_len(grib, idx);

    // if bits_per_values is null, there is no data in the sector, return it as is
    if g_p.bits_per_values == 0 {
        debug!("NO VALUES IN SECT 7");
        return Ok(grib[idx..idx + sect_len].to_vec());
    }

    // Verification that the requested area is within the provided grib
    if g_p.latitude_max > g_p.lat_end.max(g_p.lat0)
        || g_p.latitude_min < g_p.lat0.min(g_p.lat_end)
        || g_p.lon0 > g_p.longitude_min
        || g_p.lon_end < g_p.longitude_max
    {
        error!(
            "latmax: {}, lat0end: {}",
            g_p.latitude_max,
            g_p.lat_end.max(g_p.lat0)
        );
        error!(
            "latmin: {}, lat0end: {}",
            g_p.latitude_min,
            g_p.lat0.min(g_p.lat_end)
        );
        error!("lonmax: {}, lon0end: {}", g_p.longitude_max, g_p.lon_end);
        error!("lonmin: {}, lon0end: {}", g_p.longitude_min, g_p.lon0);
        return Err("Grib does not contain all the requested area".into());
    }

    debug!("g_param: {:?}", g_p);
    let params = AecParams::new(
        g_p.bits_per_values,
        g_p.block_size as u32,
        g_p.rsi as u32,
        flags_from_grib2_ccsds_flags(g_p.ccsds_flag),
    );
    let decoded = match decode(
        &grib[idx + 5..idx + sect_len],
        params,
        g_p.nb_values as usize,
    ) {
        Ok(d) => d,
        Err(e) => return Err(format!("Err decoding sect 7 : {}", e).into()),
    };

    // Verify that the payload is the expected size
    let by_p_v = g_p.bits_per_values.div_ceil(8) as usize;
    if decoded.len() != by_p_v * g_p.nb_values as usize {
        return Err(format!(
            "Decoded payload size is wrong: {}, expected {}",
            decoded.len(),
            by_p_v * g_p.nb_values as usize
        )
        .into());
    }

    // Keep only the values in the cropped area
    let mut res: Vec<u8> = Vec::new();
    let nj_o = ((g_p.lon_end - g_p.lon0) * 4.0).round() as usize + 1;
    let ni = ((g_p.latitude_max - g_p.latitude_min) * 4.0).round() as usize + 1;
    let nj =
        ((g_p.longitude_max - g_p.longitude_min) * 4.0).round() as usize + 1;
    let nb_skipped_lines = if g_p.lat0 > g_p.lat_end {
        ((g_p.lat0 - g_p.latitude_max) * 4.0).round() as usize
    } else {
        ((g_p.latitude_min - g_p.lat0) * 4.0).round() as usize
    };
    let nb_skipped_col =
        ((g_p.longitude_min - g_p.lon0) * 4.0).round() as usize;
    idx = nj_o * nb_skipped_lines * by_p_v;
    for _ in 0..ni {
        res.extend(
            &decoded[idx + nb_skipped_col * by_p_v
                ..idx + (nb_skipped_col + nj) * by_p_v],
        );
        idx += nj_o * by_p_v;
    }

    // Set the sector length
    let mut sect_len = vec![0u8; 4];
    set_4bytes(&mut sect_len, 0, res.len() + 5);

    Ok(sect_len.into_iter().chain([0x07]).chain(res).collect())
}
