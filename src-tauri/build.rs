use std::process::Command;

fn main() {
    // The commit and build date reach the About page and every Diagnostics Bundle
    // manifest, so a maintainer can check out the exact source a report came from and
    // match the right .pdb. Both fall back to "unknown" rather than failing the build:
    // a clone without git history is a legitimate way to build wgm.
    let commit = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|commit| commit.trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=WGM_BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=WGM_BUILD_DATE={}", build_date());

    // Without this, a build from a new commit reuses the old value.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-env-changed=WGM_BUILD_DATE");

    embed_app_manifest();

    // The default `WindowsAttributes` embeds tauri's own manifest as a resource, which
    // reaches only the app binary. `embed_app_manifest` links wgm.manifest into every
    // target instead, and two manifests in one binary is a hard linker error
    // (CVT1100), so tauri's copy is switched off here rather than merged.
    let attributes = tauri_build::Attributes::new();

    #[cfg(windows)]
    let attributes =
        attributes.windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());

    tauri_build::try_build(attributes).expect("failed to run tauri-build")
}

/// Give **every** binary this crate produces an application manifest — the app and
/// each `cargo test` harness alike.
///
/// tao, wry and rfd statically import `SetWindowSubclass`, `DefSubclassProc`,
/// `RemoveWindowSubclass` and `TaskDialogIndirect`, all of which are exported only by
/// comctl32 **version 6**. A process gets version 6 from WinSxS by asking for it in a
/// manifest; without one it loads 5.82 from System32 and fails to start with
/// `STATUS_ENTRYPOINT_NOT_FOUND` before `main` runs — which is what a test binary with
/// no manifest does, silently and with no useful diagnostic.
///
/// `rustc-link-arg` rather than `rustc-link-arg-tests`: the latter covers integration
/// tests only, and the lib's own unit-test binary is not one.
#[cfg(windows)]
fn embed_app_manifest() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("wgm.manifest");

    println!("cargo:rerun-if-changed=wgm.manifest");
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    // LNK4098: "defaultlib conflicts". Emitted by the manifest tool merging inputs.
    println!("cargo:rustc-link-arg=/ignore:4098");
}

#[cfg(not(windows))]
fn embed_app_manifest() {}

/// UTC, second precision, ISO-8601. Deliberately not local: this is a property of the
/// build machine, not of the user reading it.
fn build_date() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or_default();

    // Civil-from-days, so the build script needs no dependency of its own.
    let days = (seconds / 86_400) as i64;
    let time_of_day = seconds % 86_400;

    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = if month <= 2 { year + 1 } else { year };

    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z",
        hour = time_of_day / 3_600,
        minute = (time_of_day % 3_600) / 60,
        second = time_of_day % 60,
    )
}
