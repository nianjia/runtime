extern crate cc;

use cc::Build;
use std::path::Path;

trait AddCxxFile {
    fn add_cxx_files(&mut self, dict: &Path, files: Vec<&'static str>) -> &mut Self;
}

impl AddCxxFile for Build {
    fn add_cxx_files(&mut self, dict: &Path, files: Vec<&'static str>) -> &mut Build {
        for file in files {
            let full_path = dict.join(file);
            println!("cargo:rerun-if-changed={}", full_path.display());
            self.file(full_path);
        }
        self
    }
}

fn main() {
    let wrapper_path = Path::new("src/llvm/wrapper");
    println!("cargo:rerun-if-changed={}", wrapper_path.display());

    let cxx_files = [
        "ArchiveWrapper.cpp",
        "PassWrapper.cpp",
        "RustWrapper.cpp",
        "Linker.cpp",
    ];

    cc::Build::new()
        .cpp(true)
        .warnings(false)
        .add_cxx_files(wrapper_path, cxx_files.to_vec())
        .compile("llvm_wrapper");
}
