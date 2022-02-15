use std::env;
use std::path::PathBuf;

const ALLOWED_FUNC_PREFIX: &[&str] = &[
    "map",
    "policy",
    "pool",
    "prune",
    "queue",
    "repo",
    "repodata",
    "selection",
    "solv",
    "solver",
    "testcase",
    "transaction",
    "dataiterator",
    "datamatcher",
    "stringpool",
];

fn main() {
    println!("cargo:rustc-link-lib=solvext");
    println!("cargo:rustc-link-lib=bz2");
    println!("cargo:rustc-link-lib=lzma");
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=zstd");
    println!("cargo:rustc-link-lib=solv");

    let include_path = PathBuf::from("/usr/include/solv");

    let bindings = bindgen::Builder::default()
        .header(include_path.join("solver.h").to_str().unwrap())
        .header(include_path.join("solverdebug.h").to_str().unwrap())
        .header(include_path.join("selection.h").to_str().unwrap())
        .header(include_path.join("knownid.h").to_str().unwrap())
        .header(include_path.join("repo_appdata.h").to_str().unwrap())
        .header(include_path.join("repo_autopattern.h").to_str().unwrap())
        .header(include_path.join("repo_comps.h").to_str().unwrap())
        .header(include_path.join("repo_content.h").to_str().unwrap())
        .header(include_path.join("repo_deltainfoxml.h").to_str().unwrap())
        .header(include_path.join("repo_helix.h").to_str().unwrap())
        .header(include_path.join("repo_products.h").to_str().unwrap())
        .header(include_path.join("repo_pubkey.h").to_str().unwrap())
        .header(
            include_path
                .join("repo_releasefile_products.h")
                .to_str()
                .unwrap(),
        )
        .header(include_path.join("repo_repomdxml.h").to_str().unwrap())
        .header(include_path.join("repo_rpmdb.h").to_str().unwrap())
        .header(include_path.join("repo_rpmmd.h").to_str().unwrap())
        .header(include_path.join("repo_solv.h").to_str().unwrap())
        .header(include_path.join("repo_susetags.h").to_str().unwrap())
        .header(include_path.join("repo_updateinfoxml.h").to_str().unwrap())
        .header(include_path.join("repo_write.h").to_str().unwrap())
        .header(include_path.join("repo_zyppdb.h").to_str().unwrap())
        .header(include_path.join("testcase.h").to_str().unwrap())
        // Extracted from libsolv-sys crate, that was desiged to be
        // used in Debian.  It will limit the amount of exported
        // symbols, as bindgen will follow the standard headers too.
        .allowlist_type("(Id|solv_knownid)")
        .allowlist_function(format!("({}).*", ALLOWED_FUNC_PREFIX.join("|")))
        .allowlist_var(".*")
        .generate()
        .expect("Unable to generate the bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
