use opencc_fmmseg::DictionaryMaxlength;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "dict-generate-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn data(&self) -> PathBuf {
        self.0.join("vendor/opencc-fmmseg/src/data")
    }
    fn populate(&self) {
        fs::create_dir_all(self.data()).unwrap();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../src/data");
        for (_, name, _, _) in ARTIFACTS {
            fs::copy(
                source.join(format!("{name}.txt")),
                self.data().join(format!("{name}.txt")),
            )
            .unwrap();
        }
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_dict-generate"))
            .current_dir(&self.0)
            .args(args)
            .output()
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const ARTIFACTS: [(&str, &str, &[u8; 8], usize); 3] = [
    ("--tofu", "CharactersTofu", b"OCTFTOFU", 9),
    ("--compat", "CJK_Compatibility_Ideographs", b"OCCOMPAT", 8),
    ("--unicode", "Unicode_Compatibility", b"OCUNICOD", 8),
];

fn success(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// Independently decode the original wire layout and compare every source row.
// Library tests also exercise these artifacts with the restored runtime decoders.
fn verify_bin(path: &Path, text: &str, magic: &[u8; 8], width: usize) {
    let bytes = fs::read(path).unwrap();
    assert_eq!(&bytes[..8], magic);
    assert_eq!(bytes[8], 1);
    let rows: Vec<_> = text
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect();
    assert_eq!(
        u32::from_le_bytes(bytes[9..13].try_into().unwrap()) as usize,
        rows.len()
    );
    assert_eq!(bytes.len(), 13 + rows.len() * width);
    for (record, row) in bytes[13..].chunks_exact(width).zip(rows) {
        let columns: Vec<_> = row.split('\t').collect();
        for i in 0..2 {
            let scalar = u32::from_le_bytes(record[i * 4..i * 4 + 4].try_into().unwrap());
            assert_eq!(
                char::from_u32(scalar).unwrap(),
                columns[i].trim().chars().next().unwrap()
            );
        }
        if width == 9 {
            assert_eq!(
                record[8],
                opencc_fmmseg::DetofuLevel::parse(columns[2]).unwrap() as u8
            );
        }
    }
}

#[test]
fn help_exposes_auxiliary_generators() {
    let fixture = Fixture::new();
    let output = fixture.run(&["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for (flag, _, _, _) in ARTIFACTS {
        assert!(help.contains(flag));
    }
}

#[test]
fn each_generator_supports_default_and_custom_output_without_main_dicts() {
    let fixture = Fixture::new();
    fixture.populate();
    for (flag, name, magic, width) in ARTIFACTS {
        let text = fs::read_to_string(fixture.data().join(format!("{name}.txt"))).unwrap();
        success(fixture.run(&[flag, "--base-dir", "does-not-exist"]));
        verify_bin(
            &fixture.data().join(format!("{name}.bin")),
            &text,
            magic,
            width,
        );
        success(fixture.run(&[flag, "--output", "custom.bin"]));
        verify_bin(&fixture.0.join("custom.bin"), &text, magic, width);
        assert!(!fixture.0.join("dictionary_maxlength.zstd").exists());
    }
}

#[test]
fn multiple_generators_use_default_outputs_and_reject_shared_output() {
    let fixture = Fixture::new();
    fixture.populate();
    let output = fixture.run(&["--tofu", "--compat", "--unicode", "--output", "shared.bin"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--output cannot be used"));
    assert!(!fixture.0.join("shared.bin").exists());
    for (_, name, _, _) in ARTIFACTS {
        assert!(!fixture.data().join(format!("{name}.bin")).exists());
    }
    success(fixture.run(&["--tofu", "--compat", "--unicode"]));
    for (_, name, magic, width) in ARTIFACTS {
        verify_bin(
            &fixture.data().join(format!("{name}.bin")),
            &fs::read_to_string(fixture.data().join(format!("{name}.txt"))).unwrap(),
            magic,
            width,
        );
    }
}

#[test]
fn unicode_bin_preserves_duplicate_order_and_astral_scalars() {
    let fixture = Fixture::new();
    fixture.populate();
    let text = "# ordered entries\n聼\t听\n𠮷\t𠮟\n聼\t聽\n";
    fs::write(fixture.data().join("Unicode_Compatibility.txt"), text).unwrap();
    success(fixture.run(&["--unicode", "--output", "ordered.bin"]));
    verify_bin(&fixture.0.join("ordered.bin"), text, b"OCUNICOD", 8);
}

#[test]
fn generation_errors_fail() {
    let fixture = Fixture::new();
    for (flag, name, _, _) in ARTIFACTS {
        assert!(!fixture.run(&[flag]).status.success());
        fixture.populate();
        assert!(!fixture
            .run(&[flag, "--output", "missing/output.bin"])
            .status
            .success());
        fs::write(fixture.data().join(format!("{name}.txt")), "invalid row").unwrap();
        assert!(!fixture.run(&[flag]).status.success());
        fs::remove_dir_all(fixture.data()).unwrap();
    }
}

#[test]
fn normal_cbor_and_zstd_generation_round_trip() {
    let fixture = Fixture::new();
    let dicts = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dicts");
    for format in ["cbor", "zstd"] {
        let path = fixture.0.join(format!("dictionary.{format}"));
        success(fixture.run(&[
            "--format",
            format,
            "--base-dir",
            dicts.to_str().unwrap(),
            "--output",
            path.to_str().unwrap(),
        ]));
        let dictionary = if format == "cbor" {
            DictionaryMaxlength::deserialize_from_cbor(&path).unwrap()
        } else {
            DictionaryMaxlength::load_cbor_compressed(path.to_str().unwrap()).unwrap()
        };
        let opencc = opencc_fmmseg::OpenCC::from_dictionary(dictionary);
        assert_eq!(opencc.convert("汉语", "s2t", false), "漢語");
    }
}
