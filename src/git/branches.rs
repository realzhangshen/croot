use std::path::Path;

use git2::Repository;

/// Information about a single git branch.
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub is_current: bool,
}

/// List all branches (local first, then remote) for the repo at `repo_root`.
pub fn list_branches(repo_root: &Path) -> Vec<BranchInfo> {
    let Ok(repo) = Repository::open(repo_root) else {
        return Vec::new();
    };

    let mut local = Vec::new();
    let mut remote = Vec::new();

    // Local branches
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Local)) {
        for branch in branches.flatten() {
            let (branch, _) = branch;
            let name = branch.name().ok().flatten().unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let is_current = branch.is_head();
            local.push(BranchInfo {
                name,
                is_remote: false,
                is_current,
            });
        }
    }

    // Remote branches
    if let Ok(branches) = repo.branches(Some(git2::BranchType::Remote)) {
        for branch in branches.flatten() {
            let (branch, _) = branch;
            let name = branch.name().ok().flatten().unwrap_or("").to_string();
            if name.is_empty() || name.ends_with("/HEAD") {
                continue;
            }
            remote.push(BranchInfo {
                name,
                is_remote: true,
                is_current: false,
            });
        }
    }

    local.sort_by(|a, b| a.name.cmp(&b.name));
    remote.sort_by(|a, b| a.name.cmp(&b.name));

    let mut result = local;
    if !remote.is_empty() {
        result.extend(remote);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_branches_on_nonexistent_repo_returns_empty() {
        let branches = list_branches(Path::new("/nonexistent/path"));
        assert!(branches.is_empty());
    }

    /// Create a temp git repo with an initial commit and return (repo, tempdir).
    fn temp_repo() -> (Repository, tempfile::TempDir) {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        // Create an initial commit so HEAD and branches exist
        {
            let sig = git2::Signature::now("test", "test@test.com").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }
        (repo, tmp)
    }

    #[test]
    fn list_branches_returns_local_branches_sorted() {
        let (repo, tmp) = temp_repo();
        let head_commit = repo.head().unwrap().peel_to_commit().unwrap();
        // Create extra branches (default branch is already "main" or "master")
        repo.branch("beta", &head_commit, false).unwrap();
        repo.branch("alpha", &head_commit, false).unwrap();

        let branches = list_branches(tmp.path());
        let local: Vec<&str> = branches
            .iter()
            .filter(|b| !b.is_remote)
            .map(|b| b.name.as_str())
            .collect();

        // Should be sorted alphabetically
        assert!(
            local.windows(2).all(|w| w[0] <= w[1]),
            "branches not sorted: {local:?}"
        );
        assert!(local.contains(&"alpha"));
        assert!(local.contains(&"beta"));
    }

    #[test]
    fn list_branches_marks_current_branch() {
        let (_repo, tmp) = temp_repo();
        let branches = list_branches(tmp.path());
        let current: Vec<_> = branches.iter().filter(|b| b.is_current).collect();
        assert_eq!(current.len(), 1, "exactly one branch should be current");
    }

    #[test]
    fn list_branches_local_before_remote() {
        let (repo, tmp) = temp_repo();
        // Simulate a remote ref by creating a remote and a reference
        repo.remote("origin", "https://example.com/repo.git")
            .unwrap();
        let head_oid = repo.head().unwrap().target().unwrap();
        repo.reference(
            "refs/remotes/origin/feature",
            head_oid,
            false,
            "test remote ref",
        )
        .unwrap();

        let branches = list_branches(tmp.path());
        // Find the index of the first remote branch
        if let Some(first_remote) = branches.iter().position(|b| b.is_remote) {
            // All branches before should be local
            assert!(
                branches[..first_remote].iter().all(|b| !b.is_remote),
                "local branches should come before remote"
            );
        }
    }
}
