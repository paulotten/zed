use git::{
    repository::{GitRepository, RepoPath},
    status::FileStatus,
};
use gpui::{Context, Entity, SharedString};

use crate::git_store::{Repository, RepositoryEvent};

pub struct FileDiff {
    pub path: RepoPath,
    pub base_text: Option<String>,
    pub status: FileStatus,
}

pub struct DiffFromBranch {
    branch: SharedString,
    repo_head: Option<SharedString>,
    branch_head: Option<SharedString>,
    merge_base: Option<SharedString>,
    repository: Entity<Repository>,
    diff: Vec<FileDiff>,
}

impl DiffFromBranch {
    pub fn new(
        repository: Entity<Repository>,
        branch: SharedString,
        cx: &mut Context<Self>,
    ) -> Self {
        let current_head = repository
            .read(cx)
            .head_commit
            .as_ref()
            .map(|commit| commit.sha.clone());
        cx.subscribe(&repository, Self::handle_repository_updates);

        let mut this = Self {
            branch,
            repository,
            repo_head: current_head,
            branch_head: None,
            merge_base: None,
            diff: vec![],
        };

        this.on_head_change(cx);
        this
    }

    fn handle_repository_updates(
        &mut self,
        repository: Entity<Repository>,
        _event: &RepositoryEvent,
        cx: &mut Context<Self>,
    ) {
        let new_head = repository
            .read(cx)
            .head_commit
            .as_ref()
            .map(|commit| &commit.sha);

        if self.repo_head.as_ref() == new_head {
            return;
        }
        self.repo_head = new_head.cloned();
        self.on_head_change(cx)
    }

    fn on_head_change(&mut self, cx: &mut Context<Self>) {
        let get_branch_head = self.repository.update(cx, |repo, cx| {
            repo.revparse_batch(vec![self.branch.clone()])
        });

        self.updating = cx.spawn(async move |this, cx| {
            let heads = get_branch_head.await??;
            let head = heads.pop().flatten();
            this.update(cx, |this, cx| {});
            Ok(())
        })
    }
}
