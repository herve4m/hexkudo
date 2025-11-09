# Update Version

- Create a branch and change to it.
- Update `version` in `meson.build` and in `Cargo.toml`.
- Add a `<release>` entry in `data/io.github.herve4m.Hexkudo.metainfo.xml.in.in`.
- Add the same entry in `CHANGELOG.rst`.
- Commit, create a PR (_Prepare for release (version 0.8.1)_ for example), and merge.
- Change to the `main` branch, pull the new contents, and delete the version branch:

  ```
  $ git checkout main
  $ git pull
  $ git branch -d <version_branch>
  ```

- Create and push the version tag:

  ```
  $ git tag 0.8.1
  $ git push origin 0.8.1
  ```

- Wait for the GitHub Actions to complete successfully.


# Publish

- Clone https://github.com/flathub/io.github.herve4m.Hexkudo
- Create a branch, such as `herve4m/v0.8.1`.
  Change to the branch.
- Edit the `io.github.herve4m.Hexkudo.yaml` file.
  Change the `url` and the `sha256` entries.
  For these entries, go to https://github.com/herve4m/hexkudo/releases.
- Commit and create a PR named `v<version>`, such as `v0.8.1`.
- Wait for the automatic build to complete.
- Merge the PR and delete the branch.
