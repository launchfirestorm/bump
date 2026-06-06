use super::*;
use crate::lang::{self, Language};
use std::fs;

#[test]
fn gen_c_semver_writes_expected_defines() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("version.h");
    let version = make_semver("v", 1, 2, 3, 0);

    lang::output_file(Language::C, &version, &output_path).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("#define VERSION_PREFIX \"v\""));
    assert!(content.contains("#define VERSION_MAJOR 1"));
    assert!(content.contains("#define VERSION_MINOR 2"));
    assert!(content.contains("#define VERSION_PATCH 3"));
    assert!(content.contains("#define VERSION_STRING \"v1.2.3\""));
    assert!(content.contains(&format!(
        "#define VERSION_TIMESTAMP \"{}\"",
        version.timestamp.last
    )));
    assert!(content.contains("#ifndef BUMP_VERSION_H"));
}

#[test]
fn gen_c_calver_writes_version_string() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("version.h");
    let version = make_calver("");

    lang::output_file(Language::C, &version, &output_path).unwrap();

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("#define VERSION_STRING \"2026.06.05\""));
    assert!(content.contains(&format!(
        "#define VERSION_TIMESTAMP \"{}\"",
        version.timestamp.last
    )));
    assert!(content.contains("#ifndef BUMP_VERSION_H"));
}

#[test]
fn gen_tmpls_c_semver_template_is_embedded() {
    let tmpl = include_str!("../templates/c/semver.h");
    assert!(tmpl.contains("#define VERSION_PREFIX \"{prefix}\""));
    assert!(tmpl.contains("https://github.com/launchfirestorm/bump"));
}
