# linear-to-pr

A pi-rust extension that converts a Linear issue into a Git branch and pull request.

## What it does

1. Looks up the Linear issue by identifier (e.g. `PROJ-123`).
2. Creates a sanitised branch name: `task/<team-key>-<number>-<issue-title-slug>`.
3. Runs `git checkout -b <branch>` and `git push -u origin <branch>`.
4. Opens a pull request via `gh` (GitHub) or `gtr` (Gitea).

## Installation

From the repository root:

```bash
pi install ./contrib/linear-to-pr
```

For a project-local install only:

```bash
pi install ./contrib/linear-to-pr -l
```

## Configuration

Set your Linear API token:

```bash
export LINEAR_API_TOKEN="lin_api_..."
```

Optional environment variables:

- `PI_LINEAR_TO_PR_PROVIDER` — force `"github"` or `"gitea"` if auto-detection fails.
- `GITHUB_TOKEN` — used by `gh` if not already authenticated.
- `GITEA_TOKEN` / `GITEA_URL` — used by `gtr`.

## Usage

### Slash command (interactive mode)

```
/linear-to-pr PROJ-123
```

### Tool call

```json
{
  "name": "linear_to_pr",
  "input": {
    "issue": "PROJ-123",
    "base": "main",
    "draft": false
  }
}
```

## Requirements

- A Git repository with an `origin` remote.
- One of the following PR creation tools in `PATH`:
  - `gh` for GitHub remotes.
  - `gtr` for Gitea remotes.
- Network access to `api.linear.app`.

## Limitations

- The extension creates the branch from the currently checked-out commit. Any uncommitted changes stay in the working tree.
- PR creation is best-effort; if no supported provider is detected, the branch is still pushed and the result tells you how to open the PR manually.
