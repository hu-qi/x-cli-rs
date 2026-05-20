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
        Some("sync") => sync(),
        _ => {
            eprintln!("usage: cargo run -p xtask -- <check|sync>");
            std::process::exit(2);
        }
    }
}

fn check() {
    let binaries = read_manifest();
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

fn sync() {
    let binaries = read_manifest();
    let names = binaries.iter().map(|b| b.name.as_str()).collect::<Vec<_>>();
    let packages = binaries
        .iter()
        .map(|b| b.package.as_str())
        .collect::<Vec<_>>();

    sync_release_workflow(&names, &packages);
    sync_install_script(&names);
    sync_install_ps1(&names);
    sync_makefile_build(&packages);

    check();
}

fn read_manifest() -> Vec<Binary> {
    let manifest = fs::read_to_string("xcli.manifest.toml")
        .unwrap_or_else(|err| fail(&format!("read xcli.manifest.toml: {err}")));
    let binaries = parse_manifest(&manifest);

    if binaries.is_empty() {
        fail("xcli.manifest.toml does not define any [[binaries]] entries");
    }

    for binary in &binaries {
        if binary.name.trim().is_empty() {
            fail("manifest binary entry is missing `name`");
        }
        if binary.package.trim().is_empty() {
            fail(&format!(
                "manifest binary `{}` is missing `package`",
                binary.name
            ));
        }
    }

    binaries
}

fn parse_manifest(content: &str) -> Vec<Binary> {
    let mut binaries = Vec::new();
    let mut current: Option<Binary> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line == "[[binaries]]" {
            if let Some(binary) = current.take() {
                binaries.push(binary);
            }
            current = Some(Binary {
                name: String::new(),
                package: String::new(),
                smoke: None,
            });
            continue;
        }

        let Some(binary) = current.as_mut() else {
            continue;
        };
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        match key {
            "name" => binary.name = parse_string(value, key),
            "package" => binary.package = parse_string(value, key),
            "smoke" => binary.smoke = Some(parse_string(value, key)),
            _ => {}
        }
    }

    if let Some(binary) = current {
        binaries.push(binary);
    }

    binaries
}

fn parse_string(value: &str, key: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() < 2 {
        fail(&format!("manifest key `{key}` must be a quoted string"));
    }
    let quote = bytes[0];
    if (quote != b'\'' && quote != b'\"') || bytes[bytes.len() - 1] != quote {
        fail(&format!("manifest key `{key}` must be a quoted string"));
    }
    value[1..value.len() - 1].to_string()
}

fn sync_release_workflow(names: &[&str], packages: &[&str]) {
    let path = ".github/workflows/release.yml";
    let mut content = read(path);
    let package_args = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    let target_expr = "${{ matrix.target }}";
    content = replace_line_containing(
        path,
        &content,
        "run: cargo build --release --locked --target",
        &format!("run: cargo build --release --locked --target {target_expr} {package_args}"),
    );

    let quoted_names = names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    content = replace_line_containing(
        path,
        &content,
        "for name in [",
        &format!("for name in [{quoted_names}]:"),
    );
    write_if_changed(path, &content);
}

fn sync_install_script(names: &[&str]) {
    let path = "install.sh";
    let content = read(path);
    let content = replace_line_starting_with(
        path,
        &content,
        "BINS=",
        &format!("BINS=\"{}\"", names.join(" ")),
    );
    write_if_changed(path, &content);
}

fn sync_install_ps1(names: &[&str]) {
    let path = "install.ps1";
    let content = read(path);
    let bins = names
        .iter()
        .map(|name| format!("\"{name}.exe\""))
        .collect::<Vec<_>>()
        .join(", ");
    let content =
        replace_line_starting_with(path, &content, "$Bins = @(", &format!("$Bins = @({bins})"));
    write_if_changed(path, &content);
}

fn sync_makefile_build(packages: &[&str]) {
    let path = "Makefile";
    let content = read(path);
    let package_args = packages
        .iter()
        .map(|package| format!("-p {package}"))
        .collect::<Vec<_>>()
        .join(" ");
    let content = replace_line_starting_with(
        path,
        &content,
        "\tcargo build --release --locked",
        &format!("cargo build --release --locked {package_args}"),
    );
    write_if_changed(path, &content);
}

fn replace_line_containing(path: &str, content: &str, needle: &str, replacement: &str) -> String {
    replace_line(
        path,
        content,
        |line| line.contains(needle),
        needle,
        replacement,
    )
}

fn replace_line_starting_with(
    path: &str,
    content: &str,
    prefix: &str,
    replacement: &str,
) -> String {
    replace_line(
        path,
        content,
        |line| line.trim_start().starts_with(prefix),
        prefix,
        replacement,
    )
}

fn replace_line<F>(path: &str, content: &str, matches: F, label: &str, replacement: &str) -> String
where
    F: Fn(&str) -> bool,
{
    let mut found = false;
    let lines = content
        .lines()
        .map(|line| {
            if matches(line) {
                found = true;
                format!("{}{}", leading_whitespace(line), replacement)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>();
    if !found {
        fail(&format!("{path} does not contain line matching {label:?}"));
    }
    finish_lines(lines, content.ends_with('\n'))
}

fn leading_whitespace(line: &str) -> &str {
    let len = line.len() - line.trim_start().len();
    &line[..len]
}

fn finish_lines(lines: Vec<String>, trailing_newline: bool) -> String {
    let mut out = lines.join("\n");
    if trailing_newline {
        out.push('\n');
    }
    out
}

fn write_if_changed(path: &str, content: &str) {
    let old = read(path);
    if old != content {
        fs::write(path, content).unwrap_or_else(|err| fail(&format!("write {path}: {err}")));
        println!("updated {path}");
    }
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
        require_contains(path, &content, &format!("\"{name}.exe\""));
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
