use git2::Repository;
use std::{
    fmt::{self, Display},
    path::Path,
};

#[derive(Debug)]
pub enum RepositoryError {
    /// The directory is ocupied and a repository cant be opend there.
    OcupiedDir,
    CantOpenDir(std::io::Error),
    LibGit2Error(git2::Error),

    /// The origin of the repo in the given folder is not the same as the one in the url.
    DifferentOrigin,
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RepositoryError::OcupiedDir => {
                write!(f, "The specified git-direcory path was ocupied.")
            }
            RepositoryError::CantOpenDir(err) => write!(
                f,
                "The specified git-direcory path could not be opend because of an io error: { }",
                err
            ),
            RepositoryError::LibGit2Error(err) => {
                write!(f, "Libgit2 encounterd an error: { }", err)
            }
            RepositoryError::DifferentOrigin => {
                write!(f, "The origin of the repo does not match the given url.")
            }
        }
    }
}

fn dir_is_empty(path: &Path) -> Result<bool, RepositoryError> {
    Ok(std::fs::read_dir(&path)
        .map_err(|err| RepositoryError::CantOpenDir(err))?
        .take(1)
        .count()
        == 0)
}

pub fn get_and_update_repocitory(
    path: &Path,
    git_url: &str,
) -> Result<Repository, RepositoryError> {
    let path = std::fs::canonicalize(path).map_err(|err| RepositoryError::CantOpenDir(err))?;

    if (&path).is_file() {
        return Err(RepositoryError::OcupiedDir);
    }

    if dir_is_empty(&path)? {
        return match Repository::clone(git_url, path) {
            Ok(repo) => Ok(repo),
            Err(err) => Err(RepositoryError::LibGit2Error(err)),
        };
    }

    // Try to open a existing git repo.
    let repo = match Repository::open(&path) {
        Ok(repo) => Ok(repo),
        Err(err) => Err(RepositoryError::LibGit2Error(err)),
    }?;

    // Figure out if the repo has same origin as the git_url
    let mut remote = match repo.find_remote("origin") {
        Ok(remote) => Ok(remote),
        Err(err) => Err(RepositoryError::LibGit2Error(err)),
    }?;

    if remote.url() != Some(git_url) {
        // Then the repo is some other git repocitory
        return Err(RepositoryError::DifferentOrigin);
    }

    // Update the local main branch to reflect the main branch of the remote git_url repo.
    match remote.fetch(&["main"], None, None) {
        Ok(_) => {}
        Err(err) => return Err(RepositoryError::LibGit2Error(err)),
    }

    // Check if local branch is outdated
    let fetch_head = match repo.find_reference("FETCH_HEAD") {
        Ok(x) => Ok(x),
        Err(err) => Err(RepositoryError::LibGit2Error(err)),
    }?;

    let fetch_commit = match repo.reference_to_annotated_commit(&fetch_head) {
        Ok(x) => Ok(x),
        Err(err) => Err(RepositoryError::LibGit2Error(err)),
    }?;

    let analysis = match &repo.merge_analysis(&[&fetch_commit]) {
        Ok(x) => Ok(x.0),
        Err(err) => Err(RepositoryError::LibGit2Error(git2::Error::new(
            err.code(),
            err.class(),
            err.message(),
        ))),
    }?;

    if analysis.is_up_to_date() {
        // This is to fix borrowing issue. Could we lookover this defman?
        return Repository::open(&path).map_err(|err| RepositoryError::LibGit2Error(err));
    } else if analysis.is_fast_forward() {
        let refname = format!("refs/heads/{}", "main");
        let mut reference = repo
            .find_reference(&refname)
            .map_err(|err| RepositoryError::LibGit2Error(err))?;
        reference
            .set_target(fetch_commit.id(), "Fast-Forward")
            .map_err(|err| RepositoryError::LibGit2Error(err))?;
        repo.set_head(&refname)
            .map_err(|err| RepositoryError::LibGit2Error(err))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|err| RepositoryError::LibGit2Error(err))?;

        // This is to fix borrowing issue. Could we lookover this defman?
        return Repository::open(&path).map_err(|err| RepositoryError::LibGit2Error(err));
    } else {
        // TODO handle other analysis states. What should we do when the crrent branch is ahead of remote?
        todo!()
    }

    fn find_last_commit(repo: &Repository) -> Result<git2::Commit, git2::Error> {
        let obj = repo.head()?.resolve()?.peel(git2::ObjectType::Commit)?;
        obj.into_commit()
            .map_err(|_| git2::Error::from_str("Couldn't find commit"))
    }
    /// Adds all the changes to the path and makes a commit for it. Returns a id for the commit. 
    pub fn add_and_commit(
        repo: &git2::Repository,
        path: &Path,
        message: &str,
    ) -> Result<git2::Oid, git2::Error> {
        let signature = git2::Signature::now("Mr. Bot", "missing")?;

        let mut index = repo.index()?;
        index.add_path(path)?;

        let oid = index.write_tree()?;
        let parent_commit = find_last_commit(&repo)?;
        let tree = repo.find_tree(oid)?;
        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;
        Ok(commit_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn it_works() {
        let git_url = "https://github.com/Miro-Andrin/testing-git2.git";
        let path = Path::new("./testing-git2");
        let res = get_and_update_repocitory(path, git_url);
        assert!(res.is_ok());

        let git_url = "https://github.com/Miro-Andrin/testing-git2.git1";
        let path = Path::new("./testing-git2");

        let res = get_and_update_repocitory(path, git_url);
        assert!(res.is_err());

        let git_url = "https://github.com/Miro-Andrin/testing-git2.git";
        let path = Path::new(".");

        let res = get_and_update_repocitory(path, git_url);
        assert!(res.is_err());
    }
}
