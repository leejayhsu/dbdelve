---
name: rebase-upstream
description:
  Only use this when explicitly asked for. This skill rebases the current branch onto the upstream branch, then runs the install script
user-invocable: true
---

- first, check if the current branch is up to date with the upstream branch. If not, rebase the current branch onto the upstream branch.
- then install-app skill to install the app into /Applications, replacing any copy already there.