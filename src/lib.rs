extern crate cc;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("gmssl")
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub struct Build {
    out_dir: Option<PathBuf>,
    target: Option<String>,
    host: Option<String>,
}

pub struct Artifacts {
    include_dir: PathBuf,
    lib_dir: PathBuf,
    bin_dir: PathBuf,
    libs: Vec<String>,
    target: String,
}

impl Build {
    pub fn new() -> Build {
        Build {
            out_dir: env::var_os("OUT_DIR").map(|s| PathBuf::from(s).join("gmssl-build")),
            target: env::var("TARGET").ok(),
            host: env::var("HOST").ok(),
        }
    }

    pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Build {
        self.out_dir = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn target(&mut self, target: &str) -> &mut Build {
        self.target = Some(target.to_string());
        self
    }

    pub fn host(&mut self, host: &str) -> &mut Build {
        self.host = Some(host.to_string());
        self
    }

    fn cmd_make(&self) -> Command {
        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        if host.contains("dragonfly")
            || host.contains("freebsd")
            || host.contains("openbsd")
            || host.contains("solaris")
            || host.contains("illumos")
        {
            Command::new("gmake")
        } else {
            Command::new("make")
        }
    }


    fn cmd_cmake(&self) -> Command {
            Command::new("cmake")
    }

    #[cfg(windows)]
    fn check_env_var(&self, var_name: &str) -> Option<bool> {
        env::var_os(var_name).map(|s| {
            if s == "1" {
                // a message to stdout, let user know asm is force enabled
                println!(
                    "{}: nasm.exe is force enabled by the \
                    'OPENSSL_RUST_USE_NASM' env var.",
                    env!("CARGO_PKG_NAME")
                );
                true
            } else if s == "0" {
                // a message to stdout, let user know asm is force disabled
                println!(
                    "{}: nasm.exe is force disabled by the \
                    'OPENSSL_RUST_USE_NASM' env var.",
                    env!("CARGO_PKG_NAME")
                );
                false
            } else {
                panic!(
                    "The environment variable {} is set to an unacceptable value: {:?}",
                    var_name, s
                );
            }
        })
    }

    #[cfg(windows)]
    fn is_nasm_ready(&self) -> bool {
        self.check_env_var("OPENSSL_RUST_USE_NASM")
            .unwrap_or_else(|| {
                // On Windows, use cmd `where` command to check if nasm is installed
                let wherenasm = Command::new("cmd")
                    .args(&["/C", "where nasm"])
                    .output()
                    .expect("Failed to execute `cmd`.");
                wherenasm.status.success()
            })
    }

    #[cfg(not(windows))]
    fn is_nasm_ready(&self) -> bool {
        // We assume that nobody would run nasm.exe on a non-windows system.
        false
    }

    pub fn build(&mut self) -> Artifacts {
        let target = &self.target.as_ref().expect("TARGET dir not set")[..];



        let host = &self.host.as_ref().expect("HOST dir not set")[..];
        let out_dir = self.out_dir.as_ref().expect("OUT_DIR not set");

        self.run_command(Command::new("ls"),format!("{:?}",out_dir).as_str());
        
        let build_dir = out_dir.join("build");
      

        let install_dir = out_dir.join("install");

        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).unwrap();
        }
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir).unwrap();
        }

        let inner_dir = build_dir.join("src");
        fs::create_dir_all(&inner_dir).unwrap();
        cp_r(&source_dir(), &inner_dir);

        let mut ls_src = Command::new("ls");
        ls_src.arg(&inner_dir);
        self.run_command(ls_src,"display_target");

        let mut display_target = Command::new("echo");
        display_target.arg(&source_dir());
        display_target.arg(&inner_dir);
        display_target.arg(&build_dir);
        display_target.arg(&install_dir);
        self.run_command(display_target,"display_target");
        
        // And finally, run the perl configure script!

        //self.run_command(configure, "configuring OpenSSL build");

        // On MSVC we use `nmake.exe` with a slightly different invocation, so
        // have that take a different path than the standard `make` below.
        if target.contains("msvc") {
            let mut build =
                cc::windows_registry::find(target, "nmake.exe").expect("failed to find nmake");
            build.arg("build_libs").current_dir(&inner_dir);
            self.run_command(build, "building OpenSSL");

            let mut install =
                cc::windows_registry::find(target, "nmake.exe").expect("failed to find nmake");
            install.arg("install_dev").current_dir(&inner_dir);
            self.run_command(install, "installing OpenSSL");
        } else {

            let mut depend = self.cmd_cmake();
            depend.arg("src").current_dir(&build_dir);
            let path = install_dir.as_path().to_str().unwrap();
            depend.arg(format!("-DCMAKE_INSTALL_PREFIX={}",path));

            self.run_command(depend, "building OpenSSL dependencies");

            let mut build = self.cmd_make();
            build.current_dir(&build_dir);
            if !cfg!(windows) {
                if let Some(s) = env::var_os("CARGO_MAKEFLAGS") {
                    build.env("MAKEFLAGS", s);
                }
            }

/* 
            if let Some(ref isysr) = ios_isysroot {
                let components: Vec<&str> = isysr.split("/SDKs/").collect();
                build.env("CROSS_TOP", components[0]);
                build.env("CROSS_SDK", components[1]);
            }
*/

            self.run_command(build, "building OpenSSL");

            let mut install = self.cmd_make();
            install.arg("install").current_dir(&build_dir);
            self.run_command(install, "installing OpenSSL");
        }

        let libs = if target.contains("msvc") {
            vec!["libssl".to_string(), "libcrypto".to_string()]
        } else {
            vec!["gmssl".to_string()]
        };

        fs::remove_dir_all(&inner_dir).unwrap();

        Artifacts {
            lib_dir: install_dir.join("lib"),
            bin_dir: install_dir.join("bin"),
            include_dir: install_dir.join("include"),
            libs: libs,
            target: target.to_string(),
        }
    }

    fn run_command(&self, mut command: Command, desc: &str) {
        println!("running {:?}", command);
        let status = command.status();

        let (status_or_failed, error) = match status {
            Ok(status) if status.success() => return,
            Ok(status) => ("Exit status", format!("{}", status)),
            Err(failed) => ("Failed to execute", format!("{}", failed)),
        };
        panic!(
            "


Error {}:
    Command: {:?}
    {}: {}


    ",
            desc, command, status_or_failed, error
        );
    }
}

fn cp_r(src: &Path, dst: &Path) {
    for f in fs::read_dir(src).unwrap() {
        let f = f.unwrap();
        let path = f.path();
        let name = path.file_name().unwrap();

        // Skip git metadata as it's been known to cause issues (#26) and
        // otherwise shouldn't be required
        if name.to_str() == Some(".git") {
            continue;
        }

        let dst = dst.join(name);
        if f.file_type().unwrap().is_dir() {
            fs::create_dir_all(&dst).unwrap();
            cp_r(&path, &dst);
        } else {
            let _ = fs::remove_file(&dst);
            fs::copy(&path, &dst).unwrap();
        }
    }
}

fn sanitize_sh(path: &Path) -> String {
    if !cfg!(windows) {
        return path.to_str().unwrap().to_string();
    }
    let path = path.to_str().unwrap().replace("\\", "/");
    return change_drive(&path).unwrap_or(path);

    fn change_drive(s: &str) -> Option<String> {
        let mut ch = s.chars();
        let drive = ch.next().unwrap_or('C');
        if ch.next() != Some(':') {
            return None;
        }
        if ch.next() != Some('/') {
            return None;
        }
        Some(format!("/{}/{}", drive, &s[drive.len_utf8() + 2..]))
    }
}

impl Artifacts {
    pub fn include_dir(&self) -> &Path {
        &self.include_dir
    }

    pub fn lib_dir(&self) -> &Path {
        &self.lib_dir
    }

    pub fn libs(&self) -> &[String] {
        &self.libs
    }

    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
        if self.target.contains("msvc") {
            println!("cargo:rustc-link-lib=user32");
        } else if self.target == "wasm32-wasi" {
            println!("cargo:rustc-link-lib=wasi-emulated-signal");
            println!("cargo:rustc-link-lib=wasi-emulated-process-clocks");
            println!("cargo:rustc-link-lib=wasi-emulated-mman");
            println!("cargo:rustc-link-lib=wasi-emulated-getpid");
        }
    }
}
