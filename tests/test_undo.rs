// Copyright 2022 The Jujutsu Authors
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

use crate::common::TestEnvironment;

pub mod common;

#[test]
fn test_undo_rewrite_with_child() {
    // Test that if we undo an operation that rewrote some commit, any descendants
    // after that will be rebased on top of the un-rewritten commit.
    let test_env = TestEnvironment::default();
    test_env.jj_cmd_success(test_env.env_root(), &["init", "repo", "--git"]);
    let repo_path = test_env.env_root().join("repo");

    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "initial"]);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "modified"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["op", "log"]);
    let op_id_hex = stdout[3..15].to_string();
    test_env.jj_cmd_success(&repo_path, &["new", "-m", "child"]);
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", "description"]);
    insta::assert_snapshot!(stdout, @r###"
    @  child
    ◉  modified
    ◉
    "###);
    test_env.jj_cmd_success(&repo_path, &["undo", &op_id_hex]);

    // Since we undid the description-change, the child commit should now be on top
    // of the initial commit
    let stdout = test_env.jj_cmd_success(&repo_path, &["log", "-T", "description"]);
    insta::assert_snapshot!(stdout, @r###"
    @  child
    ◉  initial
    ◉
    "###);
}
#[test]
fn test_git_push_undo() {
    let test_env = TestEnvironment::default();
    let git_repo_path = test_env.env_root().join("git-repo");
    git2::Repository::init_bare(git_repo_path).unwrap();
    test_env.jj_cmd_success(test_env.env_root(), &["git", "clone", "git-repo", "repo"]);
    let repo_path = test_env.env_root().join("repo");

    test_env.jj_cmd_success(&repo_path, &["branch", "create", "main"]);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v1"]);
    test_env.jj_cmd_success(&repo_path, &["git", "push"]);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v2"]);
    test_env.jj_cmd_success(&repo_path, &["git", "push"]);
    // test_env.jj_cmd_success(&repo_path, &["git", "export"]);
    // test_env.jj_cmd_success(&repo_path, &["git", "import"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "6d2a7fcfa1a307ed2d2fec3e74e0c10b0f497edf418171b3a4a55b8b711058b1d51d792861e3d58a538653b7c6b7c9e5a843086cb493c5b3b0d1e7f619b5aecb",
        ),
        parents: [
            OperationId(
                "2b46bcfe10cc5c46eea27e64cbef436dd4d058cf1cd0ca95d39d0031c9b97f725f11527226e3a3d8a1753c87e9554c278b97feacaf55b48a9b5fa0b1eb904690",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147912000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147912000,
                ),
                tz_offset: 420,
            },
            description: "push current branch(es) to git remote origin",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj git push",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["undo"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "f60b204c090433d6db21b6ff006d3b6005782534b5d190f1f19a3ade0b01f8f0d817e3fa8e59a895d03549a2999c62efcd6433a1b3dddcb925822cab51db97b8",
        ),
        parents: [
            OperationId(
                "387fcf88f4f9adfa4bb2af5af65ac214f006c1aa61e5daad43b677546d130f8b4d8c26d59234e5c5e977f3bf065966754a2acdf5344508d76340fba6a91d9cbc",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147914000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147914000,
                ),
                tz_offset: 420,
            },
            description: "undo operation 387fcf88f4f9adfa4bb2af5af65ac214f006c1aa61e5daad43b677546d130f8b4d8c26d59234e5c5e977f3bf065966754a2acdf5344508d76340fba6a91d9cbc",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj undo",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "367d4f2f6deb0f71e3b45489f9a5bb28224562f9",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "367d4f2f6deb0f71e3b45489f9a5bb28224562f9",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["git", "export"]);
    test_env.jj_cmd_success(&repo_path, &["git", "import"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "8e17420fe2268763f296bdfa7bacc941583c043cd0d75380b025ab1b9b2acd06dbbb5ce06f07a2fa398bd9f84d93a4c7ca147902c59917a91812be1f57e21e95",
        ),
        parents: [
            OperationId(
                "9bdf69c75e86ae4eb1b2bad05c135e23a026275463e236a83910c5a72d08335bc1e2d8972c08b95d5fb30c067840b50b8ed3f016dd599a779130b3cfc0923d8f",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147917000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147917000,
                ),
                tz_offset: 420,
            },
            description: "import git refs",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj git import",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: Some(
            Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        ),
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v3"]);
    test_env.jj_cmd_success(&repo_path, &["git", "export"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "3b6be070f8a5ab61ea639862a1d925f8cadc0ccc7f40f47d66e5ba281a9881654783f908a1bd836148f0264c3e03a460595a0ac885fe3b5f0a7f6f289f1d8570",
        ),
        parents: [
            OperationId(
                "38ad141d8bf4b0113c9d75ee2687105c3034a75d0534ca6d1ca8385139e3ceb93f54543881df2ad5e611acaa91f07f255d219786db86c20c66cbe79edeaacddd",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147920000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147920000,
                ),
                tz_offset: 420,
            },
            description: "export git refs",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj git export",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: Some(
            Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        ),
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["git", "fetch"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "3b6be070f8a5ab61ea639862a1d925f8cadc0ccc7f40f47d66e5ba281a9881654783f908a1bd836148f0264c3e03a460595a0ac885fe3b5f0a7f6f289f1d8570",
        ),
        parents: [
            OperationId(
                "38ad141d8bf4b0113c9d75ee2687105c3034a75d0534ca6d1ca8385139e3ceb93f54543881df2ad5e611acaa91f07f255d219786db86c20c66cbe79edeaacddd",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147920000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147920000,
                ),
                tz_offset: 420,
            },
            description: "export git refs",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj git export",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: Some(
            Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        ),
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "2e7c7b87a7b4277a1ffa842d6cb9f9925dbc5178",
            ),
        },
    }
    "###);
    // TODO: This should probably not be considered a conflict. It currently is
    // because the undo made us forget that the remote was at v2, so the fetch
    // made us think it updated from v1 to v2 (instead of the no-op it could
    // have been).
    insta::assert_snapshot!(get_branch_output(&test_env, &repo_path), @r###"
    main: 2e7c7b87a7b4 v3
      @origin (ahead by 1 commits, behind by 1 commits): ebba8fecca7e v2
    "###);
}

// This test is currently *identical* to the previous one, except the repo it's
// operating it is colocated.
//
// Currently, this give an identical result. However, a follow-up commit will
// make the two tests behave differently.
#[test]
fn test_git_push_undo_colocated() {
    let test_env = TestEnvironment::default();
    let git_repo_path = test_env.env_root().join("git-repo");
    git2::Repository::init_bare(git_repo_path.clone()).unwrap();
    let repo_path = test_env.env_root().join("clone");
    git2::Repository::clone(git_repo_path.to_str().unwrap(), &repo_path).unwrap();
    test_env.jj_cmd_success(&repo_path, &["init", "--git-repo=."]);

    test_env.jj_cmd_success(&repo_path, &["branch", "create", "main"]);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v1"]);
    test_env.jj_cmd_success(&repo_path, &["git", "push"]);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v2"]);
    test_env.jj_cmd_success(&repo_path, &["git", "push"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "0bdbf569932d6c35f605efcdb15997ef07fe40702776d13ddba0cd657726236b9a82905f19c0969aeaee456442c3f3673045ce845fed1947942a76946849ca34",
        ),
        parents: [
            OperationId(
                "1a517642ff2dddb6024ac2dda88e7de6ca6bd31823f77a08497afbba4b06ce0a03568f542a0bbddf22de78e090650b6708fb4c9b8c8be9b4121ad1b2bd444952",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147912000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147912000,
                ),
                tz_offset: 420,
            },
            description: "push current branch(es) to git remote origin",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj git push",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["undo"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "0bdbf569932d6c35f605efcdb15997ef07fe40702776d13ddba0cd657726236b9a82905f19c0969aeaee456442c3f3673045ce845fed1947942a76946849ca34",
        ),
        parents: [
            OperationId(
                "3cbcea858e09a5bbd355a3156bd1a50b18d5d4f7c3e73452ea35dd5a44af727536dc0e142362c72c9e78e9f072b3cf62e48de16fe0bc946451aca6b1dea9cd80",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147915000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147915000,
                ),
                tz_offset: 420,
            },
            description: "import git refs",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj debug operation",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "ebba8fecca7e65141a97f3fbc265451560aa8235",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["describe", "-m", "v3"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "4e8c3a8ced721ca963b5c68bd4c3a873d9a02767472e8c53842f8d3842bc08bac4bc619d2332fea371f3e970cd0a51e4480d483ece6dac8c8733e5ce61625ffe",
        ),
        parents: [
            OperationId(
                "29cdf5a74e38740cd50bb4314619daa25c778527140f88f4db5029da39bc234b81045b11fef0ff0ae3d195761d51c612981ba3484b401032e57f833a0296fe2b",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147916000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147916000,
                ),
                tz_offset: 420,
            },
            description: "describe commit ebba8fecca7e65141a97f3fbc265451560aa8235",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj describe -m v3",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "29f0efc9eb741adc923a96619e00ff6cf63b9573",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "29f0efc9eb741adc923a96619e00ff6cf63b9573",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "29f0efc9eb741adc923a96619e00ff6cf63b9573",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "29f0efc9eb741adc923a96619e00ff6cf63b9573",
            ),
        },
    }
    "###);
    test_env.jj_cmd_success(&repo_path, &["git", "fetch"]);
    insta::assert_snapshot!(get_debug_op(&test_env, &repo_path), @r###"
    Operation {
        view_id: ViewId(
            "4e8c3a8ced721ca963b5c68bd4c3a873d9a02767472e8c53842f8d3842bc08bac4bc619d2332fea371f3e970cd0a51e4480d483ece6dac8c8733e5ce61625ffe",
        ),
        parents: [
            OperationId(
                "29cdf5a74e38740cd50bb4314619daa25c778527140f88f4db5029da39bc234b81045b11fef0ff0ae3d195761d51c612981ba3484b401032e57f833a0296fe2b",
            ),
        ],
        metadata: OperationMetadata {
            start_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147916000,
                ),
                tz_offset: 420,
            },
            end_time: Timestamp {
                timestamp: MillisSinceEpoch(
                    981147916000,
                ),
                tz_offset: 420,
            },
            description: "describe commit ebba8fecca7e65141a97f3fbc265451560aa8235",
            hostname: "host.example.com",
            username: "test-username",
            tags: {
                "args": "jj describe -m v3",
            },
        },
    }
    View {
        head_ids: {
            CommitId(
                "29f0efc9eb741adc923a96619e00ff6cf63b9573",
            ),
        },
        public_head_ids: {
            CommitId(
                "0000000000000000000000000000000000000000",
            ),
        },
        branches: {
            "main": BranchTarget {
                local_target: Some(
                    Normal(
                        CommitId(
                            "29f0efc9eb741adc923a96619e00ff6cf63b9573",
                        ),
                    ),
                ),
                remote_targets: {
                    "origin": Normal(
                        CommitId(
                            "ebba8fecca7e65141a97f3fbc265451560aa8235",
                        ),
                    ),
                },
            },
        },
        tags: {},
        git_refs: {
            "refs/heads/main": Normal(
                CommitId(
                    "29f0efc9eb741adc923a96619e00ff6cf63b9573",
                ),
            ),
            "refs/remotes/origin/main": Normal(
                CommitId(
                    "ebba8fecca7e65141a97f3fbc265451560aa8235",
                ),
            ),
        },
        git_head: None,
        wc_commit_ids: {
            WorkspaceId(
                "default",
            ): CommitId(
                "29f0efc9eb741adc923a96619e00ff6cf63b9573",
            ),
        },
    }
    "###);
    // TODO: This should probably not be considered a conflict. It currently is
    // because the undo made us forget that the remote was at v2, so the fetch
    // made us think it updated from v1 to v2 (instead of the no-op it could
    // have been).
    insta::assert_snapshot!(get_branch_output(&test_env, &repo_path), @r###"
    main: 29f0efc9eb74 v3
      @origin (ahead by 1 commits, behind by 1 commits): ebba8fecca7e v2
    "###);
}

fn get_branch_output(test_env: &TestEnvironment, repo_path: &Path) -> String {
    test_env.jj_cmd_success(repo_path, &["branch", "list"])
}

fn get_debug_op(test_env: &TestEnvironment, repo_path: &Path) -> String {
    test_env.jj_cmd_success(repo_path, &["debug", "operation"])
}
