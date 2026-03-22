use std::{fs, process::Command, thread, time::Duration};

#[test]
fn gfs_1() {
    let mut command = vec!["-m", "gfs025", "-s", "1h", "-d", "7"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "-17:-16", "-l", "-150:-149"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn gfs_2() {
    let mut command = vec!["-m", "gfs050", "-s", "1h", "-d", "33"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "-1.18:1.33", "-l", "5.25:6"]);
    command.extend(&["-o", "/tmp"]);
    cli_call(command);
}

#[test]
fn gfs_3() {
    let mut command = vec!["-m", "gfs100", "-s", "1h", "-d", "8"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "-1.18:1.33", "-l", "5.25:6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arpege025() {
    let mut command = vec!["-m", "arpege025", "-s", "1h", "-d", "5"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "43:44", "-l", "5:6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arpege100() {
    thread::sleep(Duration::from_mins(1));
    let mut command = vec!["-m", "arpege100", "-s", "1h", "-d", "5"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "43:44", "-l", "5:6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome() {
    let mut command = vec!["-m", "arome", "-s", "1h", "-d", "1"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "43:44", "-l", "5:6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome_polynesie() {
    let mut command = vec!["-m", "arome-polynesie", "-s", "12h", "-d", "1"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "-17:-16", "-l", "-150:-149"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome_guyane() {
    let mut command = vec!["-m", "arome-guyane", "-s", "6h", "-d", "1"];
    command.extend(&["-c", "wind-gust"]);
    command.extend(&["-L", "5:6", "-l", "-53:-52"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome_antille() {
    let mut command = vec!["-m", "arome-antille", "-s", "3h", "-d", "1"];
    command.extend(&["-c", "wind-gust"]);
    command.extend(&["-L", "16.33:17", "-l", "-62:-61.33"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome_ncaledonie() {
    let mut command = vec!["-m", "arome-ncaledonie", "-s", "1h", "-d", "1"];
    command.extend(&["-c", "wind-gust"]);
    command.extend(&["-L", "-23:-22", "-l", "166:167"]);
    command.extend(&["-o", ".."]);
    cli_call(command);
}

#[test]
fn arome_indien() {
    let mut command = vec!["-m", "arome-indien", "-s", "12h", "-d", "2"];
    command.extend(&["-c", "wind-gust"]);
    command.extend(&["-L", "-21:-20", "-l", "55:56"]);
    command.extend(&["-o", "/tmp"]);
    cli_call(command);
}

#[test]
fn arome0025() {
    let mut command = vec!["-m", "arome0025", "-s", "12h", "-d", "12"];
    command.extend(&["-c", "wind-gust"]);
    command.extend(&["-L", "43:44", "-l", "5:6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

fn cli_call(args: Vec<&str>) {
    thread::sleep(Duration::from_secs(5));
    let output = Command::new(env!("CARGO_BIN_EXE_grib2sail-cli"))
        .args(args.clone())
        .output()
        .expect("Failed to execute CLI");

    assert!(
        output.status.success(),
        "Command failed with stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let folder_path = args[args.len() - 1];
    let entries = fs::read_dir(folder_path).expect("Folder not found");
    let mut found = false;
    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(args[1])
                    && file_name.ends_with(".grib2")
                {
                    let metadata =
                        fs::metadata(&path).expect("Failed to get metadata");
                    assert!(metadata.len() > 0, "Grib file is empty");
                    fs::remove_file(&path).expect("Failed to delete file");
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(found, "No grib file written");
}
