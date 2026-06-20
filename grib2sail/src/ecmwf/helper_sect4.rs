use crate::core::GribError;
use crate::ecmwf::unpack_crop::{get_sect_len, set_4bytes};

pub fn sect_4(grib: &[u8], idx: usize) -> Result<Vec<u8>, GribError> {
    let sect_len = get_sect_len(grib, idx);
    let mut res: Vec<u8> = grib[idx..idx + sect_len].to_vec();

    // If template is 8, make sure timeIncrement is not set to avoid bugs
    if sect_len >= 58 && grib[idx + 8] == 0x08 {
        res[53] = 0xff;
        set_4bytes(&mut res, 54, 0);
    }
    Ok(res)
}
