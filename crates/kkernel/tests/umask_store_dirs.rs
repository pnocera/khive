#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::Command;

// A group-writable umask must not produce store directories that khive's own
// trust checks then refuse.
#[test]
fn group_writable_umask_creates_store_dirs_khive_accepts() {
    let home = tempfile::tempdir().expect("isolated home");
    std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let db = home.path().join("fresh").join("sub").join("khive.db");
    let mut command = Command::new(env!("CARGO_BIN_EXE_kkernel"));
    command
        .args(["exec", "--db"])
        .arg(&db)
        .arg("stats()")
        .current_dir(home.path())
        .env("HOME", home.path())
        .env("KHIVE_NO_DAEMON", "1");
    // SAFETY: umask is async-signal-safe.
    unsafe {
        command.pre_exec(|| {
            libc::umask(0o002);
            Ok(())
        });
    }
    let output = command.output().expect("run kkernel");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for dir in [
        home.path().join("fresh"),
        home.path().join("fresh").join("sub"),
        home.path().join(".khive"),
    ] {
        let mode = std::fs::metadata(&dir).unwrap().permissions().mode();
        assert_eq!(mode & 0o022, 0, "{} is {:o}", dir.display(), mode);
    }
}
