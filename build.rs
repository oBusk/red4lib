use cmake::Config;
use std::{
    env,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::Command,
};

// print build script logs
macro_rules! p {
  ($($tokens: tt)*) => {
      println!("cargo:warning={}", format!($($tokens)*))
  }
}

const TARGET_NAME: &str = "kraken_static";

fn main() {
    // Rebuild when these change
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=tools/decompress_kark.cpp");
    println!("cargo:rerun-if-changed=kraken/");
    println!("cargo:rerun-if-changed=WolvenKit/WolvenKit.Common/Resources/usedhashes.kark");

    // cmake config — build kraken_static and decompress_kark from top-level CMakeLists.txt
    let mut cfg = Config::new(".");
    let dst = cfg.build();

    // logging
    let cmake_profile: String = cfg.get_profile().to_owned();
    let rust_profile = std::env::var("PROFILE").unwrap();
    p!("CMAKE_PROFILE : {}", cmake_profile);
    p!("RUST_PROFILE : {}", rust_profile);
    p!("DST: {}", dst.display());

    // link kraken_static (installed to {dst}/lib/)
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static={}", TARGET_NAME);

    // decompress usedhashes.kark and generate CSV
    let out_dir = env::var("OUT_DIR").unwrap();
    let csv_path = PathBuf::from(format!("{}/metadata-resources.csv", out_dir));

    if csv_path.exists() {
        p!("file exists: {}", csv_path.display());
    } else {
        let kark_path = "WolvenKit/WolvenKit.Common/Resources/usedhashes.kark";
        let txt_path = format!("{}/usedhashes.txt", out_dir);

        // find the decompress_kark executable (installed to {dst}/bin/)
        let exe_name = if cfg!(windows) {
            "decompress_kark.exe"
        } else {
            "decompress_kark"
        };
        let decompress_exe = format!("{}/bin/{}", dst.display(), exe_name);
        p!("running: {} {} {}", decompress_exe, kark_path, txt_path);

        let status = Command::new(&decompress_exe)
            .args([kark_path, &txt_path])
            .status()
            .expect("failed to run decompress_kark");
        if !status.success() {
            panic!("decompress_kark failed with status: {}", status);
        }

        // generate CSV from decompressed text
        p!("generating CSV: {}", csv_path.display());
        generate_csv(&txt_path, csv_path.to_str().unwrap());
    }
}

/// Compute FNV-1a 64-bit hash
fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Read decompressed text file (one resource path per line), compute FNV1a64
/// hashes, and write CSV in the same format as the old metadata-resources.csv.
fn generate_csv(txt_path: &str, csv_path: &str) {
    let data = fs::read(txt_path).expect("failed to read decompressed file");
    let reader = BufReader::new(&data[..]);

    // Sort lines, mainly to align with the old metadata-resources.csv
    let mut lines: Vec<String> = BufRead::lines(reader)
        .map_while(Result::ok)
        .map(|l| l.trim().to_owned())
        .filter(|l| !l.is_empty())
        .collect();
    lines.sort();

    let mut csv = String::new();
    for line in &lines {
        let hash = fnv1a64(line.as_bytes());
        csv.push_str(&format!("{},{}\n", line, hash));
    }

    fs::write(csv_path, csv).expect("failed to write csv");
    p!("wrote {} bytes to {}", fs::metadata(csv_path).unwrap().len(), csv_path);
}
