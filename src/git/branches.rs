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
}
