use git::{
    repository::{CommitDetails, GitRepository, RepoPath},
    status::FileStatus,
};
use git2::Diff;
use gpui::{Context, Entity, SharedString};

use crate::git_store::{Repository, RepositoryEvent};

pub struct FileDiff {
    pub path: RepoPath,
    pub base_text: Option<String>,
    pub status: FileStatus,
}

pub struct DiffToDefaultBranch {
    current_head: Option<SharedString>,
    merge_base: Option<SharedString>,
    repository: Entity<Repository>,
    diff: Vec<FileDiff>,
}

impl DiffToDefaultBranch {
    pub fn new(repository: Entity<Repository>, cx: &Context<Self>) -> Self {
        let current_head = repository
            .read(cx)
            .head_commit
            .as_ref()
            .map(|commit| commit.sha.clone());

        cx.subscribe(&repository, Self::handle_repository_updates);
        Self {
            repository,
            current_head,
            merge_base: None,
            diff: vec![],
        }
    }

    fn handle_repository_updates(
        &mut self,
        _repository: &Repository,
        _event: &RepositoryEvent,
        cx: &mut Context<Self>,
    ) {
        // Handle repository updates here
    }
}
