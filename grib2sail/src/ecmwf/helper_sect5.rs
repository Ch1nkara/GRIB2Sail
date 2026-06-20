use crate::core::GribError;
use crate::ecmwf::unpack_crop::{GribParams, get_sect_len, set_4bytes};

pub fn sect_5(
    grib: &[u8],
    idx: usize,
    g_p: &mut GribParams,
) -> Result<Vec<u8>, GribError> {
    let sect_len = get_sect_len(grib, idx);

    // Verification that the rest of sector is as expected
    if sect_len != 25 {
        return Err(format!("Sect5 wrong size: {}", sect_len).into());
    }
    if grib[idx + 9] != 0x00 || grib[idx + 10] != 0x2A {
        return Err(format!(
            "Unexpected template: 0x{:02X}{:02X}, not 5.42 (0x002A)",
            grib[idx + 9],
            grib[idx + 10]
        )
        .into());
    }

    // Parse the parameters needed to unpack the data
    let nb_values =
        [grib[idx + 5], grib[idx + 6], grib[idx + 7], grib[idx + 8]];
    g_p.nb_values = u32::from_be_bytes(nb_values);
    g_p.bits_per_values = grib[idx + 19];
    g_p.ccsds_flag = grib[idx + 21];
    g_p.block_size = grib[idx + 22];
    let rsi = [grib[idx + 23], grib[idx + 24]];
    g_p.rsi = u16::from_be_bytes(rsi);

    let ni = ((g_p.latitude_max - g_p.latitude_min) * 4.0).round() as usize + 1;
    let nj =
        ((g_p.longitude_max - g_p.longitude_min) * 4.0).round() as usize + 1;

    // Create the new sect5, template: simple packing
    let mut res: Vec<u8> = grib[idx..idx + 21].to_vec();
    // Update the sector length
    set_4bytes(&mut res, 0, 21);
    // Update the number of values
    set_4bytes(&mut res, 5, ni * nj);
    // Update template to simple packing: 0x00
    res[10] = 0x00;
    // Update the bits_per_values according to the unpacking librairy:
    // The closest mutiple of 8 that is higher than current bits_per_values
    res[19] = grib[idx + 19].div_ceil(8) * 8;

    Ok(res)
}
