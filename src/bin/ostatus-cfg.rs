use std::fs;
use std::path;

use git2::Repository;

fn clone(url: &str, dir: impl AsRef<path::Path>) -> ostatus::GenericResult<Repository> {
    Ok(git2::Repository::clone(url, dir)?)
}

fn checkout(repo: &Repository, refname: &str) -> ostatus::GenericResult<()> {
    let (object, reference) = repo.revparse_ext(refname)?;
    repo.checkout_tree(&object, None)?;
    match reference {
        Some(gref) => repo.set_head(gref.name().unwrap()),
        None => repo.set_head_detached(object.id()),
    }?;

    Ok(())
}

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

fn roles_and_patterns(element: &xmltree::Element) -> ostatus::GenericResult<ostatus::Roles> {
    let mut roles = ostatus::Roles::default();

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
        let installation = ostatus::ReferenceInstallation {
            patterns: default_patterns(system_role).unwrap_or_default(),
            ..ostatus::ReferenceInstallation::default()
        };
        roles.0.insert(role, installation);
    }

    Ok(roles)
}

fn read_control(control: impl AsRef<path::Path>) -> ostatus::GenericResult<ostatus::Roles> {
    let control = fs::File::open(control)?;
    let root = xmltree::Element::parse(control)?;

    let default = ostatus::ReferenceInstallation {
        patterns: default_patterns(&root).unwrap_or_default(),
        // For now we set some well known defaults
        packages_opt: vec![
            "kernel-default".to_string(),
            "kernel-pae".to_string(),
            "kernel-vanilla".to_string(),
            "snapper".to_string(),
            "grub2".to_string(),
            "btrfsprogs".to_string(),
        ],
        ..ostatus::ReferenceInstallation::default()
    };

    let mut roles = roles_and_patterns(&root)?;
    roles.0.insert("default".to_string(), default);

    Ok(roles)
}

fn roles_to_config(roles: ostatus::Roles) -> String {
    let mut config = String::new();

    let mut role_names: Vec<_> = roles.0.keys().collect();
    role_names.sort();

    for role in role_names {
        config.push_str(&format!("[{}]\n", role));

        let installation = roles.0.get(role).unwrap();

        if !installation.patterns.is_empty() {
            config.push_str(&format!("patterns = {}\n", installation.patterns.join(" ")));
        }

        if !installation.packages.is_empty() {
            config.push_str(&format!("packages = {}\n", installation.packages.join(" ")));
        }
        if !installation.patterns_opt.is_empty() {
            config.push_str(&format!(
                "patterns_opt = {}\n",
                installation.patterns_opt.join(" ")
            ));
        }
        if !installation.packages_opt.is_empty() {
            config.push_str(&format!(
                "packages_opt = {}\n",
                installation.packages_opt.join(" ")
            ));
        }
        config.push('\n');
    }

    config
}

fn run() -> ostatus::GenericResult<()> {
    let projects = vec![
        (
            "openSUSE",
            vec![
                ("openSUSE-15_3", "opensuse-leap-15.3"),
                ("openSUSE-15_4", "opensuse-leap-15.4"),
                ("master", "opensuse-tumbleweed"),
            ],
        ),
        ("MicroOS", vec![("master", "opensuse-microos")]),
        (
            "SMO",
            vec![
                ("SLE-Micro-5.1", "suse-microos-5.1"),
                ("SLE-Micro-5.2", "suse-microos-5.2"),
            ],
        ),
    ];

    for (name, branches) in projects {
        let url = format!("https://github.com/yast/skelcd-control-{}.git", name);
        let dir = tempfile::tempdir()?;
        let repo = clone(&url, dir.path())?;

        for (branch, id_version) in branches {
            let product = format!("{}/{}", name, branch);
            println!("Configuration for {}", product);

            checkout(&repo, &format!("origin/{}", branch))?;

            let control = format!(
                "{}/control/control.{}.xml",
                dir.path().to_str().unwrap(),
                name
            );

            let config = roles_to_config(read_control(&control)?);

            let filename = format!("{}.cfg", id_version);
            println!("Creating {}", filename);
            fs::write(&filename, config)?;
        }
    }

    Ok(())
}

fn main() {
    std::process::exit(match run() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("error: {}", e);
            1
        }
    });
}
