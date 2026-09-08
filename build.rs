use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=logo.png");
    println!("cargo:rerun-if-changed=assets/logo.ico");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let rc_path = out_dir.join("app.rc");
        let res_path = out_dir.join("app.res");

        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ico_path = manifest_dir.join("assets").join("logo.ico");
        let rc_content = format!(
            "1 ICON \"{}\"\n",
            ico_path.to_str().unwrap().replace('\\', "\\\\")
        );
        let _ = std::fs::write(&rc_path, rc_content);

        let mut compiled = false;
        let rc_candidates = [
            PathBuf::from("rc.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\rc.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\rc.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.22000.0\x64\rc.exe"),
            PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\rc.exe"),
            PathBuf::from(r"C:\ProgramData\mingw64\mingw64\bin\windres.exe"),
        ];

        for candidate in &rc_candidates {
            let is_windres = candidate.to_string_lossy().contains("windres");
            let mut cmd = Command::new(candidate);
            if is_windres {
                cmd.args(["-i", rc_path.to_str().unwrap(), "-O", "coff", "-o", res_path.to_str().unwrap()]);
            } else {
                cmd.args(["/fo", res_path.to_str().unwrap(), rc_path.to_str().unwrap()]);
            }

            if let Ok(status) = cmd.status() {
                if status.success() {
                    compiled = true;
                    break;
                }
            }
        }

        if compiled {
            println!("cargo:rustc-link-arg={}", res_path.to_str().unwrap());
        }
    }
}
