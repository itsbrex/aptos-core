// Copyright © Aptos Labs
// SPDX-License-Identifier: Apache-2.0

//! Test artifacts: examples known to have crashed in the past.

use aptos_fuzzer::FuzzTarget;
use rusty_fork::{fork, rusty_fork_id};
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::{alloc::System, env, fs, path::Path};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

/// The memory limit for each deserializer, in bytes.
const MEMORY_LIMIT: usize = 256 * 1024 * 1024;

datatest_stable::harness!(test_artifact, "artifacts", r"^.*/.*");

fn test_artifact(artifact_path: &Path) -> datatest_stable::Result<()> {
    let test_name = test_name(artifact_path);

    if no_fork() {
        test_artifact_impl(artifact_path);
    } else {
        fork(
            &test_name,
            rusty_fork_id!(),
            |_| {},
            |child, _file| {
                let status = child.wait().expect("failed to wait for child");
                assert!(
                    status.success(),
                    "child exited unsuccessfully with {}",
                    status
                );
            },
            || test_artifact_impl(artifact_path),
        )?;
    }

    Ok(())
}

fn no_fork() -> bool {
    env::var_os("NO_FORK").map_or(false, |x| x == "1")
}

fn test_artifact_impl(artifact_path: &Path) {
    // Extract the target from the path -- it's the second component after "artifacts/".
    let target_name = artifact_path
        .iter()
        .nth(1)
        .expect("artifact path must be in format 'artifacts/<target>/<filename>'");
    let target_name = target_name.to_str().expect("target must be valid Unicode");
    let target = FuzzTarget::by_name(target_name)
        .unwrap_or_else(|| panic!("unknown fuzz target: {}", target_name));
    let data = fs::read(artifact_path).expect("failed to read artifact");

    let reg = Region::new(GLOBAL);
    target.fuzz(&data);
    let stats = reg.change();

    eprintln!("stats: {:?}", stats);
    assert!(
        stats.bytes_allocated <= MEMORY_LIMIT,
        "Deserializer used too much memory: allocated {} bytes (max {} bytes)",
        stats.bytes_allocated,
        MEMORY_LIMIT
    );
}

fn test_name(artifact_path: &Path) -> String {
    // This matches the test name generated by datatest.
    let mut test_name = "test_artifact::".to_string();

    let path = artifact_path
        .strip_prefix("artifacts/")
        .expect("artifact path doesn't begin with artifacts/");
    let subname = path.to_str().expect("name must be valid unicode");
    test_name.push_str(subname);
    test_name
}
