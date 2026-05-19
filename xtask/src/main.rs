use std::{env, fs, path::Path};

#[derive(Debug, Clone)]
struct Binary {
    name: String,
    package: String,
    smoke: Option<String>,
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("check") => check(),
        Some("sync") => {
            eprintln!("xtask sync is not implemented yet; run `xtask check` for now");
            std::process::exit(2);
        }
        _ => {
            eprintln!("usage: cargo run -p xtask -- <check|sync>");
            std::process::exit(2);
        }
    }
}

fn check() {
    let manifest = fs::read_to_string("xcli.manifest.toml")
        .unwrap_or_else(|err| fail(&format!("read xcli.manifest.toml: {err}")));
    let binaries = parse_manifest(&manifest);
    if binaries.is_empty() {
        fail("xcli.manifest.toml does not define any [[binaries]] entries");
    }

    let names = binaries.iter().map(|b| b.name.as_str()).collect::<Vec<_>>();
    let packages = binaries
        .iter()
        .map(|b| b.package.as_str())
        .collect::<Vec<_>>();

    check_root_cargo(&packages);
    check_release_workflow(&names, &packages);
    check_install_script(&names);
    check_install_ps1(&names);
    check_makefile(&packages, &binaries);
    check_readme("README.md", &names);
    check_readme("README-zh.md", &names);
    check_release_checklist(&names);

    println!("xtask check passed: {} binaries are in sync", names.len());
}

fn parse_manifest(input: &str) -> Vec<Binary> {
    let mut out = Vec::new();
    let mut current: Option<Binary> = None;

    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[binaries]]" {
            if let Some(binary) = current.take() {
                out.push(binary);
            }
            current = Some(Binary {
                name: String::new(),
                package: String::new(),
                smoke: None,
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        let Some(binary) = current.as_mut() else {
            continue;
        };

        match key {
            "name" => binary.name = parse_string(value),
            "package" => binary.package = parse_string(value),
            "smoke" => binary.smoke = Some(parse_string(value)),
            _ => {}
        }
    }

    if let Some(binary) = current.take() {
        out.push(binary);
    }

    for binary in &out {
        if binary.name.is_empty() {
            fail("manifest binary entry is missing `name`");
        }
        if binary.package.is_empty() {
            fail(&format!("manifest binary `{}` is missing `package`", binary.name));
        }
    }

    out
}

fn parse_string(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return value[1..value.len() - 1].to_string();
        }
    }
    value.to_string()
}

fn check_root_cargo(packages: &[&str]) {
    let content = read("Cargo.toml");
    for package in packages {
        let member = if *package == "xcli" {
            "crates/xcli".to_string()
        } else if package.ends_with("-cli") {
            format!("examples/{package}")
        } else {
            continue;
        };
        require_contains("Cargo.toml", &content, &format!("\"{member}\""));
    }
}

fn check_release_workflow(names: &[&str], packages: &[&str]) {
    let path = ".github/workflows/release.yml";
    let content = read(path);
    for package in packages {
        require_contains(path, &content, &format!("-p {package}"));
    }
    for name in names {
        require_contains(path, &content, &format!("\"{name}\""));
    }
}

fn check_install_script(names: &[&str]) {
    let path = "install.sh";
    let content = read(path);
    for name in names {
        require_contains(path, &content, name);
    }
}

fn check_install_ps1(names: &[&str]) {
    let path = "install.ps1";
    let content = read(path);
    for name in names {
        require_contains(path, &content, &format!("\"{name}\""));
    }
}

fn check_makefile(packages: &[&str], binaries: &[Binary]) {
    let path = "Makefile";
    let content = read(path);
    for package in packages {
        require_contains(path, &content, &format!("-p {package}"));
    }
    for binary in binaries {
        if let Some(smoke) = &binary.smoke {
            require_contains(path, &content, smoke);
        }
    }
}

fn check_readme(path: &str, names: &[&str]) {
    let content = read(path);
    for name in names {
        require_contains(path, &content, name);
    }
}

fn check_release_checklist(names: &[&str]) {
    let path = "docs/release-checklist.md";
    let content = read(path);
    for name in names {
        require_contains(path, &content, name);
    }
}

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(path)).unwrap_or_else(|err| fail(&format!("read {path}: {err}")))
}

fn require_contains(path: &str, haystack: &str, needle: &str) {
    if !haystack.contains(needle) {
        fail(&format!("{path} is missing manifest entry: {needle}"));
    }
}

fn fail(message: &str) -> ! {
    eprintln!("xtask check failed: {message}");
    std::process::exit(1);
}
