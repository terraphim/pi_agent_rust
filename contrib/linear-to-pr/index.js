/**
 * Pi extension: linear-to-pr
 *
 * Converts a Linear issue into a Git branch and opens a pull request.
 *
 * Usage:
 *   In interactive mode: /linear-to-pr PROJ-123
 *   As a tool: linear_to_pr({ issue: "PROJ-123", base: "main", draft: false })
 */

const LINEAR_API_URL = "https://api.linear.app/graphql";

function log(level, message, data) {
  if (typeof pi.log === "function") {
    pi.log({
      level,
      event: "linear-to-pr",
      message,
      data,
    });
  }
}

function env(key) {
  if (pi.env && typeof pi.env.get === "function") {
    return pi.env.get(key);
  }
  return undefined;
}

function sanitizeBranchName(text) {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 60)
    .replace(/-+$/, "");
}

function makeBranchName(teamKey, issueNumber, title) {
  const slug = sanitizeBranchName(title || "linear-issue");
  return `task/${teamKey.toLowerCase()}-${issueNumber}-${slug}`;
}

async function httpPostJson(url, headers, body) {
  const response = await pi.http({
    url,
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });

  if (response.status < 200 || response.status >= 300) {
    throw new Error(`Linear API returned ${response.status}: ${response.body}`);
  }

  return JSON.parse(response.body || "{}");
}

async function fetchLinearIssue(token, identifier) {
  const query = `
    query IssueByIdentifier($identifier: String!) {
      issues(filter: { identifier: { eq: $identifier } }) {
        nodes {
          id
          identifier
          title
          description
          url
          state { name }
          team { key }
        }
      }
    }
  `;

  const result = await httpPostJson(
    LINEAR_API_URL,
    {
      "Content-Type": "application/json",
      Authorization: token,
    },
    { query, variables: { identifier } }
  );

  if (result.errors && result.errors.length > 0) {
    throw new Error(
      `Linear API error: ${result.errors.map((e) => e.message).join(", ")}`
    );
  }

  const nodes = result.data?.issues?.nodes || [];
  if (nodes.length === 0) {
    throw new Error(`Linear issue not found: ${identifier}`);
  }

  return nodes[0];
}

async function execOrFail(cmd, args, options = {}) {
  log("debug", `exec: ${cmd} ${args.join(" ")}`);
  const result = await pi.exec(cmd, args, options);

  if (result.code !== 0) {
    const stderr = result.stderr || "";
    const stdout = result.stdout || "";
    throw new Error(
      `Command failed: ${cmd} ${args.join(" ")}\nexit ${result.code}\n${stdout}${stderr}`
    );
  }

  return result.stdout || "";
}

async function getGitRemote(name = "origin") {
  try {
    const output = await execOrFail("git", ["remote", "get-url", name]);
    return output.trim();
  } catch (err) {
    log("warn", `Could not read git remote ${name}`, { error: String(err) });
    return null;
  }
}

function parseRepoFromRemote(url) {
  if (!url) return null;

  const httpsMatch = url.match(/https?:\/\/[^/]+\/([^/]+)\/([^/]+?)(?:\.git)?$/);
  if (httpsMatch) {
    return { owner: httpsMatch[1], repo: httpsMatch[2].replace(/\.git$/, "") };
  }

  const sshMatch = url.match(/git@[^:]+:([^/]+)\/([^/]+?)(?:\.git)?$/);
  if (sshMatch) {
    return { owner: sshMatch[1], repo: sshMatch[2].replace(/\.git$/, "") };
  }

  return null;
}

async function detectGitProvider() {
  const configured = env("PI_LINEAR_TO_PR_PROVIDER");
  if (configured) return configured;

  const remote = await getGitRemote();
  if (!remote) return null;

  if (remote.includes("github.com")) return "github";
  if (remote.includes("git.terraphim.cloud") || remote.includes("gitea")) {
    return "gitea";
  }

  return null;
}

async function createGitHubPr(owner, repo, branch, base, title, body, draft) {
  const args = [
    "pr", "create",
    "--repo", `${owner}/${repo}`,
    "--base", base,
    "--head", branch,
    "--title", title,
    "--body", body || "",
  ];
  if (draft) args.push("--draft");

  const url = await execOrFail("gh", args);
  return { provider: "github", url: url.trim() };
}

async function createGiteaPr(owner, repo, branch, base, title, body, draft) {
  const args = [
    "create-pull",
    "--owner", owner,
    "--repo", repo,
    "--title", title,
    "--base", base,
    "--head", branch,
  ];
  if (body) args.push("--body", body);
  if (draft) args.push("--draft");

  const output = await execOrFail("gtr", args);
  return { provider: "gitea", url: output.trim() };
}

async function createPullRequest(provider, owner, repo, branch, base, title, body, draft) {
  switch (provider) {
    case "github":
      return createGitHubPr(owner, repo, branch, base, title, body, draft);
    case "gitea":
      return createGiteaPr(owner, repo, branch, base, title, body, draft);
    default:
      throw new Error(
        `Unsupported Git provider: ${provider}. Set PI_LINEAR_TO_PR_PROVIDER to "github" or "gitea".`
      );
  }
}

async function runLinearToPr({ issue: issueIdentifier, base = "main", draft = false }) {
  if (!issueIdentifier) {
    throw new Error("Missing required argument: issue (e.g. PROJ-123)");
  }

  const token = env("LINEAR_API_TOKEN");
  if (!token) {
    throw new Error(
      "LINEAR_API_TOKEN is not set. Add it to your environment or pi auth storage."
    );
  }

  const issue = await fetchLinearIssue(token, issueIdentifier);
  const teamKey = issue.team?.key || "linear";
  const issueNumber = issue.identifier.split("-").pop();
  const branchName = makeBranchName(teamKey, issueNumber, issue.title);

  const remote = await getGitRemote();
  const repoInfo = parseRepoFromRemote(remote);
  const provider = await detectGitProvider();

  log("info", `Ensuring branch ${branchName} for ${issue.identifier}`);
  try {
    await execOrFail("git", ["checkout", branchName]);
  } catch {
    await execOrFail("git", ["checkout", "-b", branchName]);
  }
  await execOrFail("git", ["push", "-u", "origin", branchName]);

  const prTitle = `[${issue.identifier}] ${issue.title}`;
  const prBody = issue.url
    ? `Closes ${issue.url}`
    : `Linear issue ${issue.identifier}`;

  let prResult = null;
  if (provider && repoInfo) {
    prResult = await createPullRequest(
      provider,
      repoInfo.owner,
      repoInfo.repo,
      branchName,
      base,
      prTitle,
      prBody,
      draft
    );
  }

  return {
    issue: {
      identifier: issue.identifier,
      title: issue.title,
      url: issue.url,
      state: issue.state?.name,
    },
    branch: branchName,
    remote,
    provider,
    pull_request: prResult,
    next_step: prResult
      ? `Review the pull request at ${prResult.url}`
      : `Create a pull request manually from branch ${branchName} to ${base}`,
  };
}

export default function init(pi) {
  pi.registerCommand("linear-to-pr", {
    description: "Convert a Linear issue into a branch and pull request",
    handler: async (args) => {
      const parts = typeof args === "string" ? args.trim().split(/\s+/) : [];
      const issue = parts.length > 0 ? parts[0] : null;
      const result = await runLinearToPr({ issue });
      return JSON.stringify(result, null, 2);
    },
  });

  pi.registerTool({
    name: "linear_to_pr",
    label: "Linear to PR",
    description:
      "Fetch a Linear issue, create a Git branch, push it, and open a pull request.",
    parameters: {
      type: "object",
      properties: {
        issue: {
          type: "string",
          description: "Linear issue identifier, e.g. PROJ-123",
        },
        base: {
          type: "string",
          description: "Base branch for the pull request",
          default: "main",
        },
        draft: {
          type: "boolean",
          description: "Create the pull request as draft",
          default: false,
        },
      },
      required: ["issue"],
    },
    execute: async (_callId, input) => {
      const result = await runLinearToPr({
        issue: input?.issue,
        base: input?.base || "main",
        draft: Boolean(input?.draft),
      });

      return {
        content: [
          {
            type: "text",
            text: `Created branch ${result.branch} for ${result.issue.identifier}. ${result.next_step}`,
          },
        ],
        details: result,
        isError: false,
      };
    },
  });
}
