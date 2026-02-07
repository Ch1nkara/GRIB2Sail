use rust_core::utils::download_file;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cli <url>");
        return;
    }
    let url = &args[1];
    let result = download_file(url);
    println!("{}", result);
}
