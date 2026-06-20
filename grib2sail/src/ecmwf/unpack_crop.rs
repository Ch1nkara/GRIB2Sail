use crate::core::{DownloadEvent, Grib, GribError};

use crate::ecmwf::helper_sect3::sect_3;
use crate::ecmwf::helper_sect4::sect_4;
use crate::ecmwf::helper_sect5::sect_5;
use crate::ecmwf::helper_sect7::sect_7;

use log::debug;
use std::{sync::Arc, thread::available_parallelism};
use tokio::{
    spawn,
    sync::{Semaphore, mpsc::UnboundedSender},
};

#[derive(Debug, Clone)]
pub struct GribParams {
    pub latitude_max: f64,
    pub latitude_min: f64,
    pub longitude_max: f64,
    pub longitude_min: f64,
    pub lat0: f64,
    pub lat_end: f64,
    pub lon0: f64,
    pub lon_end: f64,
    pub nb_values: u32,
    pub bits_per_values: u8,
    pub ccsds_flag: u8,
    pub block_size: u8,
    pub rsi: u16,
}

pub async fn unpack_crop(
    grib: Grib,
    events: UnboundedSender<DownloadEvent>,
) -> Result<Grib, GribError> {
    let g_param = GribParams {
        latitude_min: (grib.latitude_min / 0.25).floor() * 0.25,
        latitude_max: (grib.latitude_max / 0.25).ceil() * 0.25,
        longitude_min: (grib.longitude_min / 0.25).floor() * 0.25,
        longitude_max: (grib.longitude_max / 0.25).ceil() * 0.25,
        lat0: 0.0,
        lat_end: 0.0,
        lon0: 0.0,
        lon_end: 0.0,
        nb_values: 0x00000000,
        bits_per_values: 0x00,
        ccsds_flag: 0x00,
        block_size: 0x00,
        rsi: 0x0000,
    };
    let mut res_grib = Grib {
        content: Vec::new(),
        ..grib.clone()
    };
    let old_content: Arc<[u8]> = Arc::from(grib.content);
    let concurrency = available_parallelism().map(|n| n.get()).unwrap_or(1);
    let semaphore = Arc::new(Semaphore::new(concurrency)); // based on CPU nb
    let mut tasks = Vec::new();
    let mut res_content = Vec::new();

    let mut idx = 0;
    let mut i = 0;
    // Handle each grib message
    while idx < old_content.len() {
        let msg_len = [
            old_content[idx + 8],
            old_content[idx + 9],
            old_content[idx + 10],
            old_content[idx + 11],
            old_content[idx + 12],
            old_content[idx + 13],
            old_content[idx + 14],
            old_content[idx + 15],
        ];
        let msg_len: usize = u64::from_be_bytes(msg_len)
            .try_into()
            .expect("Failed to convert u64 to usize");
        tasks.push((
            i,
            spawn(unpack_crop_message(
                Arc::clone(&old_content),
                idx,
                msg_len,
                g_param.clone(),
                events.clone(),
                Arc::clone(&semaphore),
            )),
        ));
        idx += msg_len;
        i += 1;
    }

    // Collect data as they are handled by individual tasks
    for task in tasks {
        res_content.push((task.0, task.1.await??))
    }

    // Restore original order
    res_content.sort_by_key(|(idx, _)| *idx);

    // Concatenate Vec<Vec<u8 into Vec<u8
    res_grib.content =
        res_content.into_iter().flat_map(|(_, data)| data).collect();
    Ok(res_grib)
}

async fn unpack_crop_message(
    old_ctt: Arc<[u8]>,
    mut idx: usize,
    msg_len: usize,
    mut g_p: GribParams,
    events: UnboundedSender<DownloadEvent>,
    semaphore: Arc<Semaphore>,
) -> Result<Vec<u8>, GribError> {
    let _permit = semaphore.acquire_owned().await?;
    events.send(DownloadEvent::FinishedOne)?;
    let mut res_msg = Vec::new();

    // Handle section 0
    debug!("Section 0");
    let msg_end = idx + msg_len;
    res_msg.extend_from_slice(&old_ctt[idx..idx + 16]);
    idx += 16;

    // Handle section 1 to 8
    while idx < msg_end {
        if msg_end - idx < 4 {
            return Err("Parse error: message too short".into());
        }
        // Detect section section 8
        if msg_end - idx == 4 {
            // 0x37 -> ASCII for 7, the end of grib message marker
            if old_ctt[idx] == 0x37
                && old_ctt[idx + 1] == 0x37
                && old_ctt[idx + 2] == 0x37
                && old_ctt[idx + 3] == 0x37
            {
                debug!("Section 8");
                res_msg.extend_from_slice(&old_ctt[idx..idx + 4]);
            } else {
                return Err("Parse error: section 8 missing 7777".into());
            }
            idx += 4;
            continue;
        }

        // Parse section number and length
        let sect_nb = old_ctt[idx + 4];
        let sect_len = get_sect_len(&old_ctt, idx);

        // Don't touch section 1, 2 and 6. Edit the others (3, 4, 5 and 7)
        match sect_nb {
            1 | 2 | 6 => {
                res_msg.extend_from_slice(&old_ctt[idx..idx + sect_len])
            }
            3 => res_msg.extend(sect_3(&old_ctt, idx, &mut g_p)?),
            4 => res_msg.extend(sect_4(&old_ctt, idx)?),
            5 => res_msg.extend(sect_5(&old_ctt, idx, &mut g_p)?),
            7 => res_msg.extend(sect_7(&old_ctt, idx, &g_p)?),
            _ => {
                return Err(format!("Unexpected section {}", sect_nb).into());
            }
        }
        idx += sect_len;
    }
    // Update the message length
    let res_msg_len = (res_msg.len() as u64).to_be_bytes();
    debug!("message length is {:?} and idx is {}", res_msg_len, idx);
    res_msg[8..16].copy_from_slice(&res_msg_len);

    Ok(res_msg)
}

pub fn get_sect_len(grib: &[u8], idx: usize) -> usize {
    let sect_len: [u8; 4] =
        [grib[idx], grib[idx + 1], grib[idx + 2], grib[idx + 3]];
    u32::from_be_bytes(sect_len) as usize
}

pub fn set_4bytes(grib: &mut [u8], idx: usize, value: usize) {
    let bytes = (value as u32).to_be_bytes();
    grib[idx..idx + 4].copy_from_slice(&bytes);
}
