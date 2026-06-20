#! /bin/sh

# First, update your remote-tracking refs:
git fetch --prune

# Loop over all local branches
for branch in $(git branch --format="%(refname:short)"); do
  # Skip the current branch (and any other branches you want to keep, e.g., main or staging)
  if [ "$branch" = "staging" ] || [ "$branch" = "main" ] || [ "$branch" = "master" ]; then
    continue
  fi

  # Check if there is a remote branch with the same name
  if ! git show-ref --verify --quiet "refs/remotes/origin/$branch"; then
    echo "Deleting branch: $branch"
    git branch -D "$branch"
  fi
done
