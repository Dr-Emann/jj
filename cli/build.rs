// Copyright 2023 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::Path;
use std::process::Command;
use std::str;

use cargo_metadata::MetadataCommand;

const GIT_HEAD_PATH: &str = "../.git/HEAD";
const JJ_OP_HEADS_PATH: &str = "../.jj/repo/op_heads/heads";

fn main() -> std::io::Result<()> {
    let path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let meta = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .current_dir(&path)
        .exec()
        .unwrap();
    let root = meta.root_package().unwrap();
    let version = &root.version;

    if Path::new(GIT_HEAD_PATH).exists() {
        // In colocated repo, .git/HEAD should reflect the working-copy parent.
        println!("cargo:rerun-if-changed={GIT_HEAD_PATH}");
    } else if Path::new(JJ_OP_HEADS_PATH).exists() {
        // op_heads changes when working-copy files are mutated, which is way more
        // frequent than .git/HEAD.
        println!("cargo:rerun-if-changed={JJ_OP_HEADS_PATH}");
    }
    println!("cargo:rerun-if-env-changed=NIX_JJ_GIT_HASH");

    if let Some((date, git_hash)) = get_git_date_and_hash() {
        println!(
            "cargo:rustc-env=JJ_VERSION={}-{}-{}",
            version, date, git_hash
        );
    } else {
        println!("cargo:rustc-env=JJ_VERSION={}", version);
    }

    Ok(())
}

/// Return the committer date in YYYYMMDD format and the git hash
fn get_git_date_and_hash() -> Option<(String, String)> {
    if let Some(nix_hash) = std::env::var("NIX_JJ_GIT_HASH")
        .ok()
        .filter(|s| !s.is_empty())
    {
        return Some(("nix".to_string(), nix_hash));
    }

    fn trim_and_split_on_vbar(bytes: &[u8]) -> (String, String) {
        let s = str::from_utf8(bytes).unwrap().trim_end();
        let (date, id) = s.split_once('|').unwrap();
        (date.to_owned(), id.to_owned())
    }
    if let Ok(output) = Command::new("jj")
        .args([
            "--ignore-working-copy",
            "--color=never",
            "log",
            "--no-graph",
            "-r=@-",
            r#"-T=committer.timestamp().format("%Y%m%d") ++ "|" ++ commit_id"#,
        ])
        .output()
    {
        if output.status.success() {
            return Some(trim_and_split_on_vbar(&output.stdout));
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["log", "-1", "--format=%cs|%H", "HEAD"])
        .output()
    {
        if output.status.success() {
            let (date, hash) = trim_and_split_on_vbar(&output.stdout);
            return Some((date.replace('-', ""), hash));
        }
    }

    None
}
