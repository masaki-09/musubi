//! End-to-end CLI tests. They invoke the actual `musubi` binary built
//! by Cargo and exercise stdin/stdout plus file I/O round-trips.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn musubi() -> Command {
    Command::cargo_bin("musubi").expect("musubi binary built")
}

#[test]
fn round_trip_plaintext_via_files() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("key.json");
    let cipher_path = dir.path().join("cipher.json");

    musubi()
        .args(["keygen", "--seed", "42"])
        .arg("-o")
        .arg(&key_path)
        .assert()
        .success();

    musubi()
        .args(["encrypt", "-k"])
        .arg(&key_path)
        .arg("-o")
        .arg(&cipher_path)
        .write_stdin("Hello, musubi!")
        .assert()
        .success();

    musubi()
        .args(["decrypt", "-k"])
        .arg(&key_path)
        .arg("-i")
        .arg(&cipher_path)
        .assert()
        .success()
        .stdout("Hello, musubi!\n");
}

#[test]
fn round_trip_japanese_with_anchor_flag() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("key.json");

    musubi()
        .args(["keygen", "--seed", "1"])
        .arg("-o")
        .arg(&key_path)
        .assert()
        .success();

    let cipher_output = musubi()
        .args(["encrypt", "-k"])
        .arg(&key_path)
        .args(["-a", "2"])
        .write_stdin("あいしてる")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    musubi()
        .args(["decrypt", "-k"])
        .arg(&key_path)
        .write_stdin(cipher_output)
        .assert()
        .success()
        .stdout("あいしてる\n");
}

#[test]
fn out_of_alphabet_chars_fail_with_helpful_message() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("key.json");

    musubi()
        .args(["keygen", "--seed", "0"])
        .arg("-o")
        .arg(&key_path)
        .assert()
        .success();

    musubi()
        .args(["encrypt", "-k"])
        .arg(&key_path)
        .write_stdin("世界")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in the alphabet"));
}

#[test]
fn empty_plaintext_fails() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("key.json");

    musubi()
        .args(["keygen", "--seed", "5"])
        .arg("-o")
        .arg(&key_path)
        .assert()
        .success();

    musubi()
        .args(["encrypt", "-k"])
        .arg(&key_path)
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn keygen_compact_seed_is_deterministic() {
    let dir = tempdir().unwrap();
    let a = dir.path().join("a.json");
    let b = dir.path().join("b.json");

    musubi()
        .args(["keygen", "--seed", "12345"])
        .arg("-o")
        .arg(&a)
        .assert()
        .success();
    musubi()
        .args(["keygen", "--seed", "12345"])
        .arg("-o")
        .arg(&b)
        .assert()
        .success();

    let a_content = std::fs::read_to_string(&a).unwrap();
    let b_content = std::fs::read_to_string(&b).unwrap();
    assert_eq!(
        a_content, b_content,
        "seeded keygen should be deterministic"
    );
}

#[test]
fn version_flag_prints_a_version() {
    musubi()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("musubi"));
}
