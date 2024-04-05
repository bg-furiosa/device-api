use std::env;
use std::path::{Path, PathBuf};
use autotools;
use hex_literal::hex;
use tar::Archive;
use sha3::{Digest, Sha3_256};
use flate2::read::GzDecoder;

fn main() {
    let version = "2.10.0";
    // This assumes you're using the hex_literal crate for the hex! macro
    let sha3_digest = hex!("b3e5e208587cd366fc2975f21102c20f3b1094d3cae69f464cb1bd8b09f302aa");
    let out_path = env::var("OUT_DIR").expect("No output directory given");

    let source_path = fetch_hwloc(out_path, version, sha3_digest);
    install_hwloc_autotools(&source_path);
}

fn fetch_hwloc(parent_path: impl AsRef<Path>, version: &str, sha3_digest: [u8; 32]) -> PathBuf {
    // Predict location where tarball would be extracted
    let parent_path = parent_path.as_ref();
    let extracted_path = parent_path.join(format!("hwloc-{version}"));

    // Reuse any existing download
    if extracted_path.exists() {
        eprintln!("Reusing previous hwloc v{version} download");
        return extracted_path;
    }

    // Determine hwloc tarball URL
    let mut version_components = version.split('.');
    let major = version_components.next().expect("no major hwloc version");
    let minor = version_components.next().expect("no minor hwloc version");
    let url = format!(
        "https://download.open-mpi.org/release/hwloc/v{major}.{minor}/hwloc-{version}.tar.gz"
    );

    // Download hwloc tarball
    eprintln!("Downloading hwloc v{version} from URL {url}...");
    let tar_gz = attohttpc::get(url)
        .send()
        .expect("failed to GET hwloc source")
        .bytes()
        .expect("failed to parse hwloc source HTTP body");

    // Verify tarball integrity
    eprintln!("Verifying hwloc source integrity...");
    let mut hasher = Sha3_256::new();
    hasher.update(&tar_gz[..]);
    assert_eq!(
        &hasher.finalize()[..],
        sha3_digest,
        "downloaded hwloc source failed integrity check"
    );

    // Extract tarball
    eprintln!("Extracting hwloc source...");
    let tar = GzDecoder::new(&tar_gz[..]);
    let mut archive = Archive::new(tar);
    archive
        .unpack(parent_path)
        .expect("failed to extract hwloc source");

    // Predict location where tarball was extracted
    extracted_path
}

fn install_hwloc_autotools(source_path: &Path) {
    let mut config = autotools::Config::new(source_path);
    config.enable_static().disable_shared();

    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "macos" {
        config.ldflag("-F/System/Library/Frameworks -framework CoreFoundation");
    }

    let install_path = config.fast_build(true).reconf("-ivf").build();

    // Construct the PKG_CONFIG_PATH to include both lib and lib64 directories
    let pkg_config_path = format!(
        "{}:{}",
        install_path.join("lib").join("pkgconfig").to_string_lossy(),
        install_path.join("lib64").join("pkgconfig").to_string_lossy(),
    );
    env::set_var("PKG_CONFIG_PATH", &pkg_config_path);

    // Now, for Unix-like systems, set the RPATH so the dynamic linker can find the hwloc libraries.
    if env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default() == "unix" {
        // Assuming hwloc libraries are installed in the 'lib' directory under the install_path.
        let rpath_dir = install_path.join("lib");
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            rpath_dir.to_string_lossy()
        );
    }
}
