use std::fs;
use std::process::Command;

#[test]
fn gfs_1() {
    let mut command = vec!["-m", "gfs", "-s", "3h", "-d", "5"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "-17,-16", "-l", "-150,-149"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

#[test]
fn arome() {
    let mut command = vec!["-m", "arome", "-s", "1h", "-d", "1"];
    command.extend(&["-c", "wind,wind-gust,pressure,cloud-cover"]);
    command.extend(&["-L", "43,44", "-l", "5,6"]);
    command.extend(&["-o", "."]);
    cli_call(command);
}

fn cli_call(args: Vec<&str>) {
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
