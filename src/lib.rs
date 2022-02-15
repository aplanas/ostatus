use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::Hash;
use std::io::{self, BufRead, Write};
use std::path;
use std::str;
use std::sync::Once;
use std::time;

use chrono::prelude::*;
use configparser::ini;
use regex::Regex;
use sha2::{Digest, Sha256};

static CONFIGURE: Once = Once::new();

pub static STATUS_DIR: &str = "/usr/lib/sysimage/ostatus";

static CONFIG_DIR_SYS: &str = "/usr/etc";
static CONFIG_DIR: &str = "/etc";
static CONFIG: &str = "ostatus.cfg";

pub type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type GenericResult<T> = Result<T, GenericError>;

struct ElementSearch<'a>(&'a xmltree::Element);

impl ElementSearch<'_> {
    fn get_child<P: xmltree::ElementPredicate>(&self, k: P) -> Option<ElementSearch> {
        self.0.get_child(k).map(ElementSearch)
    }

    fn get_children<P: xmltree::ElementPredicate>(
        &self,
        k: P,
    ) -> impl Iterator<Item = &xmltree::Element> {
        self.0
            .children
            .iter()
            .filter_map(|e| match e {
                xmltree::XMLNode::Element(elem) => Some(elem),
                _ => None,
            })
            .filter(move |e| k.match_element(e))
    }
}

/// An installation of reference
#[derive(Default, Debug)]
struct ReferenceInstallation {
    patterns: Vec<String>,
    packages: Vec<String>,
    patterns_opt: Vec<String>,
    packages_opt: Vec<String>,
}

impl ReferenceInstallation {
    fn from_ini(cfg: &ini::Ini, section: &str) -> ReferenceInstallation {
        ReferenceInstallation {
            patterns: cfg
                .get(section, "patterns")
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            packages: cfg
                .get(section, "packages")
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            patterns_opt: cfg
                .get(section, "patterns_opt")
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            packages_opt: cfg
                .get(section, "packages_opt")
                .unwrap_or_default()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Roles(HashMap<String, ReferenceInstallation>);

impl Roles {
    pub fn from_config(paths: &[impl AsRef<path::Path>]) -> GenericResult<Roles> {
        let mut all_cfgs = String::new();

        for path in paths {
            all_cfgs.push_str(&fs::read_to_string(path)?);
        }

        let mut cfg = ini::Ini::new();
        cfg.read(all_cfgs)?;

        let mut roles = Roles::default();
        roles.0.insert(
            "default".to_string(),
            ReferenceInstallation::from_ini(&cfg, "default"),
        );

        for section in cfg.sections() {
            if section != "default" {
                roles.0.insert(
                    section.clone(),
                    ReferenceInstallation::from_ini(&cfg, &section),
                );
            }
        }

        Ok(roles)
    }

    pub fn from_control(path: impl AsRef<path::Path>) -> GenericResult<Roles> {
        let control = fs::File::open(path)?;
        let root = xmltree::Element::parse(control)?;

        let default = ReferenceInstallation {
            patterns: Roles::default_patterns(&root).unwrap_or_default(),
            // For now we set some well known defaults
            packages_opt: vec![
                "kernel-default".to_string(),
                "kernel-pae".to_string(),
                "kernel-vanilla".to_string(),
                "snapper".to_string(),
                "grub2".to_string(),
                "btrfsprogs".to_string(),
            ],
            ..ReferenceInstallation::default()
        };

        let mut roles = Roles::roles_patterns(&root)?;
        roles.0.insert("default".to_string(), default);

        Ok(roles)
    }

    fn default_patterns(element: &xmltree::Element) -> Option<Vec<String>> {
        Some(
            element
                .get_child("software")?
                .get_child("default_patterns")?
                .get_text()?
                .split(' ')
                .map(|s| s.to_owned())
                .collect::<Vec<String>>(),
        )
    }

    fn roles_patterns(element: &xmltree::Element) -> GenericResult<Roles> {
        let mut roles = Roles(HashMap::new());

        let product_defines_search = ElementSearch(element);
        for system_role in product_defines_search
            .get_child("system_roles")
            .unwrap()
            .get_children("system_role")
        {
            let role = system_role
                .get_child("id")
                .unwrap()
                .get_text()
                .unwrap()
                .into_owned();
            let installation = ReferenceInstallation {
                patterns: Roles::default_patterns(system_role).unwrap_or_default(),
                ..ReferenceInstallation::default()
            };
            roles.0.insert(role, installation);
        }

        Ok(roles)
    }

    pub fn apply_default(&mut self) {
        if let Some(default) = self.0.remove("default") {
            for installation in self.0.values_mut() {
                if installation.patterns.is_empty() {
                    installation.patterns = default.patterns.clone();
                }
                if installation.packages.is_empty() {
                    installation.packages = default.packages.clone();
                }
                if installation.patterns_opt.is_empty() {
                    installation.patterns_opt = default.patterns_opt.clone();
                }
                if installation.packages_opt.is_empty() {
                    installation.packages_opt = default.packages_opt.clone();
                }
            }
        }
    }
}

pub fn find_configs() -> GenericResult<Vec<String>> {
    let release = OsRelease::new()?;

    let mut filenames = Vec::new();
    filenames.push(format!("{}.cfg", release.id));
    filenames.push(format!("{}-{}.cfg", release.id, release.version_id));
    filenames.push(CONFIG.to_string());

    let mut configs = Vec::new();
    for dir in [CONFIG_DIR_SYS, CONFIG_DIR] {
        for filename in &filenames {
            let config = format!("{}/{}", dir, filename);
            if path::Path::new(&config).exists() {
                configs.push(config);
            }
        }
    }

    Ok(configs)
}

fn baseproduct() -> GenericResult<String> {
    let baseproduct = fs::File::open("/etc/products.d/baseproduct")?;
    let product = xmltree::Element::parse(baseproduct)?;
    let baseproduct = product
        .get_child("name")
        .expect("Product name not found")
        .get_text()
        .unwrap()
        .into_owned();
    Ok(baseproduct)
}

#[derive(Debug)]
struct ZypperRepo {
    pub alias: String,
    pub priority: i64,
}

fn repo_alias() -> GenericResult<Vec<ZypperRepo>> {
    let mut repos = Vec::new();
    let mut urls = HashSet::new();

    let files = fs::read_dir("/etc/zypp/repos.d")?;
    for repo_fn in files {
        let path = repo_fn?.path();
        if let Some(extension) = path.extension() {
            if extension == "repo" {
                let mut config = ini::Ini::new();
                config.load(path)?;
                for alias in config.sections() {
                    let solv = format!("/var/cache/zypp/solv/{}/solv", alias);
                    if !path::Path::new(&solv).exists() {
                        continue;
                    }
                    let priority = config
                        .getint(&alias, "priority")
                        .unwrap_or(None)
                        .unwrap_or(99);
                    let enabled = config
                        .getboolcoerce(&alias, "enabled")
                        .unwrap_or(None)
                        .unwrap_or(true);
                    if !enabled {
                        continue;
                    }
                    // TODO what if there are many
                    let url = config
                        .get(&alias, "baseurl")
                        .unwrap_or_else(|| "".to_string());
                    if !urls.contains(&url) {
                        urls.insert(url);
                        repos.push(ZypperRepo { alias, priority });
                    }
                }
            }
        }
    }

    repos.sort_by(|a, b| a.priority.cmp(&b.priority));
    Ok(repos)
}

struct OsRelease {
    _name: String,
    id: String,
    version_id: String,
    _pretty_name: String,
}

impl OsRelease {
    pub fn new() -> GenericResult<OsRelease> {
        let mut config = ini::Ini::new();
        config.load("/etc/os-release")?;

        Ok(OsRelease {
            _name: config
                .get("default", "NAME")
                .expect("NAME not found in os-release")
                .replace('"', ""),
            id: config
                .get("default", "ID")
                .expect("ID not found in os-release")
                .replace('"', ""),
            version_id: config
                .get("default", "VERSION_ID")
                .expect("VERSION_ID not found in os-release")
                .replace('"', ""),
            _pretty_name: config
                .get("default", "PRETTY_NAME")
                .expect("PRETTY_NAME not found in os-release")
                .replace('"', ""),
        })
    }
}

#[derive(Debug)]
struct Installation {
    products: Vec<libsolv_rs::pool::Package>,
    patterns: Vec<libsolv_rs::pool::Package>,
    packages: Vec<libsolv_rs::pool::Package>,
}

impl Installation {
    fn autoinstalled() -> GenericResult<HashSet<String>> {
        let mut autoinst = HashSet::new();
        let autoinst_file = io::BufReader::new(fs::File::open("/var/lib/zypp/AutoInstalled")?);
        for line in autoinst_file.lines().flatten() {
            if line.starts_with('#') {
                continue;
            }
            autoinst.insert(line);
        }
        Ok(autoinst)
    }

    fn from_system_no_autoinstalled() -> GenericResult<Installation> {
        let autoinst = Installation::autoinstalled()?;

        let mut products = Vec::new();
        let mut patterns = Vec::new();
        let mut packages = Vec::new();

        let system = io::BufReader::new(fs::File::open("/var/cache/zypp/solv/@System/solv.idx")?);
        for line in system.lines().flatten() {
            let element: Vec<&str> = line.split_whitespace().collect();
            if let [name, version, arch] = element[..] {
                if autoinst.contains(name) {
                    continue;
                }
                let version = version.to_owned();
                let arch = arch.to_owned();
                let type_name: Vec<&str> = name.split(':').collect();
                match type_name[..] {
                    ["product", name] => products.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    ["pattern", name] => patterns.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    [name] => packages.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    _ => (),
                };
            }
        }

        Ok(Installation {
            products,
            patterns,
            packages,
        })
    }

    fn from_system() -> GenericResult<Installation> {
        let mut products = Vec::new();
        let mut patterns = Vec::new();
        let mut packages = Vec::new();

        let system = io::BufReader::new(fs::File::open("/var/cache/zypp/solv/@System/solv.idx")?);
        for line in system.lines().flatten() {
            let element: Vec<&str> = line.split_whitespace().collect();
            if let [name, version, arch] = element[..] {
                let version = version.to_owned();
                let arch = arch.to_owned();
                let type_name: Vec<&str> = name.split(':').collect();
                match type_name[..] {
                    ["product", name] => products.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    ["pattern", name] => patterns.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    [name] => packages.push(libsolv_rs::pool::Package {
                        name: name.to_owned(),
                        version,
                        arch,
                    }),
                    _ => (),
                };
            }
        }

        Ok(Installation {
            products,
            patterns,
            packages,
        })
    }

    fn from_role(role: &str, roles: &Roles) -> GenericResult<Installation> {
        let repo_alias = repo_alias()?;
        let products = vec![baseproduct()?];
        let ref_installation = roles.0.get(role).expect("Role not found");
        let extra_packages = Vec::new();
        let test_case = testcase(
            &repo_alias,
            &products,
            &ref_installation.patterns,
            &extra_packages,
        )?;

        let mut pool = libsolv_rs::pool::Pool::new();
        let installables = pool.testsolv(&test_case);

        let mut products = Vec::new();
        let mut patterns = Vec::new();
        let mut packages = Vec::new();

        for installable in installables.into_iter() {
            let type_name: Vec<&str> = installable.name.split(':').collect();
            let version = installable.version;
            let arch = installable.arch;
            match type_name[..] {
                ["product", name] => products.push(libsolv_rs::pool::Package {
                    name: name.to_owned(),
                    version,
                    arch,
                }),
                ["pattern", name] => patterns.push(libsolv_rs::pool::Package {
                    name: name.to_owned(),
                    version,
                    arch,
                }),
                [name] => packages.push(libsolv_rs::pool::Package {
                    name: name.to_owned(),
                    version,
                    arch,
                }),
                _ => (),
            };
        }

        Ok(Installation {
            products,
            patterns,
            packages,
        })
    }
}

fn jaccard<T>(set1: &HashSet<T>, set2: &HashSet<T>) -> f64
where
    T: Eq + Hash,
{
    let union = set1.union(set2);
    let intersection = set1.intersection(set2);
    intersection.count() as f64 / union.count() as f64
}

fn find_closer_role(roles: &Roles, installation: &Installation) -> GenericResult<Option<String>> {
    let installed_patterns_set: HashSet<_> =
        installation.patterns.iter().map(|p| &p.name).collect();

    let mut best_role = None;
    let mut best_index = 0.0;
    for (role, ref_installation) in &roles.0 {
        let patterns: HashSet<_> = ref_installation.patterns.iter().collect();
        let index = jaccard(&installed_patterns_set, &patterns);
        if index > best_index {
            best_index = index;
            best_role = Some(role.clone());
        }
    }

    Ok(best_role)
}

#[derive(Debug)]
pub struct ZypperConf {
    pub only_requires: bool,
    pub allow_vendor_change: bool,
}

impl ZypperConf {
    pub fn new() -> GenericResult<ZypperConf> {
        let conf = fs::read_to_string("/etc/zypp/zypp.conf")?;
        let re_only_requires = Regex::new(r"(?m)^solver.onlyRequires\s*=\s*true")?;
        let re_allow_vendor_change = Regex::new(r"(?m)^solver.allowVendorChange\s*=\s*true")?;

        Ok(ZypperConf {
            only_requires: re_only_requires.is_match(&conf),
            allow_vendor_change: re_allow_vendor_change.is_match(&conf),
        })
    }
}

fn testcase(
    repo_alias: &[ZypperRepo],
    products: &[String],
    patterns: &[String],
    packages: &[String],
) -> GenericResult<String> {
    let mut repos = "".to_string();
    for repo in repo_alias {
        repos.push_str(&format!(
            "repo {alias} {prio} solv /var/cache/zypp/solv/{alias}/solv\n",
            alias = repo.alias,
            prio = repo.priority
        ));
    }

    let mut flags = Vec::new();
    let zypper_conf = ZypperConf::new()?;
    if zypper_conf.only_requires {
        flags.push("ignorerecommended");
    }
    if zypper_conf.allow_vendor_change {
        flags.push("allowvendorchange");
    }
    if !flags.is_empty() {
        flags.insert(0, "solverflags");
        flags.push("\n");
    }
    let flags = flags.join(" ");

    let mut jobs = Vec::new();
    for product in products {
        jobs.push(format!("job install name product:{}", product));
    }
    for pattern in patterns {
        jobs.push(format!("job install name pattern:{}", pattern));
    }
    for package in packages {
        jobs.push(format!("job install name {}", package));
    }
    let jobs = jobs.join("\n");

    let testcase = format!(
        "system {arch} rpm\n\n{repos}\n{flags}{jobs}",
        arch = std::env::consts::ARCH,
        repos = repos,
        flags = flags,
        jobs = jobs
    );

    Ok(testcase)
}

fn buildtime_from_repos(repos: &[String]) -> HashMap<String, u64> {
    let mut buildtimes = HashMap::new();

    let mut pool = libsolv_rs::pool::Pool::new();
    let mut repo = libsolv_rs::repo::Repo::new(&mut pool, "local solv repos");
    for alias in repos {
        repo.add_solv(
            &format!("/var/cache/zypp/solv/{}/solv", alias),
            libsolv_rs::repo::RepoFlags::empty(),
        );
    }
    for repoid in 1..pool.nrepos() {
        let r = pool.repo(repoid).unwrap();
        for solvableid in r.start()..r.end() {
            let mut solvable = pool.solvable(solvableid).unwrap();
            buildtimes.insert(solvable.nevra(), solvable.buildtime());
        }
    }

    buildtimes
}

fn configure() {
    CONFIGURE.call_once(|| {
        librpm::config::read_file(None).unwrap();
    });
}

fn buildtime_from_system() -> HashMap<String, time::SystemTime> {
    let mut buildtimes = HashMap::new();

    configure();
    for p in librpm::db::installed_packages() {
        buildtimes.insert(p.nevra(), p.buildtime());
    }

    buildtimes
}

fn base_manifest(installation: &Installation, status_dir: &str) -> GenericResult<()> {
    let repo_alias: Vec<String> = repo_alias()?.into_iter().map(|r| r.alias).collect();
    let buildtimes = buildtime_from_repos(&repo_alias);

    let mut doc = Vec::new();
    for product in &installation.products {
        doc.push(format!("product:{}", product.full_name()));
    }
    for pattern in &installation.patterns {
        doc.push(format!("pattern:{}", pattern.full_name()));
    }
    for package in &installation.packages {
        doc.push(format!(
            "{} {}",
            package.full_name(),
            buildtimes.get(&package.full_name()).unwrap()
        ));
    }

    doc.sort();
    let mut manifest = fs::File::create(&format!("{}/base.manifest", status_dir))?;
    manifest.write_all(doc.join("\n").as_bytes())?;

    Ok(())
}

fn system_manifest(installation: &Installation, status_dir: &str) -> GenericResult<()> {
    let buildtimes = buildtime_from_system();

    let mut doc = Vec::new();
    for product in &installation.products {
        doc.push(format!("product:{}", product.full_name()));
    }
    for pattern in &installation.patterns {
        doc.push(format!("pattern:{}", pattern.full_name()));
    }
    for package in &installation.packages {
        if buildtimes.contains_key(&package.full_name()) {
            let buildtime = buildtimes.get(&package.full_name()).unwrap();

            doc.push(format!(
                "{} {}",
                package.full_name(),
                buildtime
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));
        } else {
            doc.push(package.full_name());
        }
    }

    doc.sort();
    let mut manifest = fs::File::create(&format!("{}/system.manifest", status_dir))?;
    manifest.write_all(doc.join("\n").as_bytes())?;

    Ok(())
}

fn gzip(file: &str) -> GenericResult<()> {
    let file_gz = format!("{}.gz", file);
    let mut encoder =
        flate2::write::GzEncoder::new(fs::File::create(&file_gz)?, flate2::Compression::default());

    encoder.write_all(fs::read_to_string(file)?.as_bytes())?;
    encoder.finish()?;

    fs::remove_file(file)?;

    Ok(())
}

fn is_prefix(element: &str, prefixes: Option<&[&str]>) -> bool {
    if let Some(prefixes) = prefixes {
        for prefix in prefixes {
            if element.starts_with(prefix) {
                return true;
            }
        }
    }
    false
}

fn diff_and_join(
    vec_a: &[libsolv_rs::pool::Package],
    vec_b: &[libsolv_rs::pool::Package],
    exclude: Option<&[&str]>,
) -> String {
    let item_set: HashSet<_> = vec_b.iter().map(|item| item.name.clone()).collect();
    let mut difference: Vec<_> = vec_a
        .iter()
        .map(|item| item.name.clone())
        .filter(|item| !item_set.contains(item) && !is_prefix(item, exclude))
        .collect();
    difference.sort();

    difference.join(" ")
}

pub fn create_status_file(roles: Roles, status_dir: &str) -> GenericResult<()> {
    if !path::Path::new(status_dir).exists() {
        fs::create_dir(status_dir)?;
    }

    let mut status = Vec::new();
    status.push(format!(r#"DATE="{}""#, Utc::now()));
    status.push(format!(r#"PRODUCT="{}""#, baseproduct()?));

    let release = OsRelease::new()?;
    status.push(format!(r#"VERSION_ID="{}""#, release.version_id));

    let inst_system = Installation::from_system()?;
    let role = find_closer_role(&roles, &inst_system)?.expect("Role cannot be detected");
    status.push(format!(r#"ROLE="{}""#, role));

    // System manifest contains the list of packages expected for the
    // role, but the buildtime from rpmdb.  Maybe should have the list
    // of installed packages.
    let inst_role = Installation::from_role(&role, &roles)?;
    system_manifest(&inst_role, status_dir)?;
    base_manifest(&inst_role, status_dir)?;

    let mut hasher = Sha256::new();
    hasher.update(fs::read_to_string(&format!(
        "{}/base.manifest",
        status_dir
    ))?);
    status.push(format!(
        r#"BASE_MANIFEST_DIGEST="{:x}""#,
        hasher.finalize_reset()
    ));

    hasher.update(fs::read_to_string(&format!(
        "{}/system.manifest",
        status_dir
    ))?);
    status.push(format!(
        r#"SYSTEM_MANIFEST_DIGEST="{:x}""#,
        hasher.finalize()
    ));

    gzip(&format!("{}/base.manifest", status_dir))?;
    gzip(&format!("{}/system.manifest", status_dir))?;

    let packages_user = Installation::from_system_no_autoinstalled()?;
    status.push(format!(
        r#"ADDED_PATTERNS="{}""#,
        diff_and_join(&packages_user.patterns, &inst_role.patterns, None)
    ));
    status.push(format!(
        r#"REMOVED_PATTERNS="{}""#,
        diff_and_join(&inst_role.patterns, &inst_system.patterns, None)
    ));

    status.push(format!(
        r#"ADDED_PACKAGES="{}""#,
        diff_and_join(
            &packages_user.packages,
            &inst_role.packages,
	    // TODO part of this is already in the config file
            Some(&["patterns-", "kernel-", "snapper", "grub2", "btrfsprogs"])
        )
    ));
    status.push(format!(
        r#"REMOVED_PACKAGES="{}""#,
        diff_and_join(&inst_role.packages, &inst_system.packages, None)
    ));


    let mut ostatus = fs::File::create(&format!("{}/ostatus", status_dir))?;
    ostatus.write_all(status.join("\n").as_bytes())?;

    Ok(())
}
