use git::{
    repository::{CommittedFile, GitRepository, RepoPath},
    status::FileStatus,
};
use gpui::{Context, Entity, SharedString, Task};
use util::ResultExt;

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
    base_texts: HashMap<RepoPath, Entity<Buffer>>,
    updating: Task<()>,
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
            repo_head: None,
            merge_base: None,
            diff: vec![],
            updating: Task::ready(()),
        };

        if let Some(current_head) = current_head {
            this.on_head_change(current_head, cx);
        }
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
        if let Some(new_head) = new_head {
            self.on_head_change(new_head.clone(), cx)
        } else {
            self.diff.clear();
            self.updating = Task::ready(());
            self.merge_base.take();
            self.repo_head.take();
        }
    }

    fn on_head_change(&mut self, new_head: SharedString, cx: &mut Context<Self>) {
        let get_merge_base = self.repository.update(cx, |repo, cx| {
            repo.merge_base("HEAD".into(), self.branch.clone())
        });

        let task = cx.spawn(async move |this, cx| {
            let merge_base = get_merge_base.await??;

            let load_branch_diff = this
                .update(cx, |this, cx| {
                    if this.repo_head == Some(new_head) && this.merge_base == Some(merge_base) {
                        return None;
                    }

                    Some(this.repository.update(cx, |repository, cx| {
                        repository.load_branch_diff(new_head.clone(), merge_base.clone())
                    }))
                })?
                .await??;

            let Some(task) = load_branch_diff else {
                return;
            }
            let diff = task.await??;

            let required = this.update(cx, |this, cx| {
                this.diff = diff;
                let mut required = Vec::new();
                for entry in &this.diff {
                    if !this.base_texts.has_key(&entry.path) {
                        required.push(CommittedFile {
                            commit: merge_base.clone(),
                            path: entry.path.clone(),
                        });
                    }
                }

                this.repository.update(cx, |repository, cx| {
                    repository.batch_file_content(required)
                })
                required
            })?;

            anyhow::Ok(())
        });

        self.updating = cx.spawn(async move |this, cx| {
            task.await.log_err();
        })
    }
}
