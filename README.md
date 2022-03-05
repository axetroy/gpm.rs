# gpm

[![ci](https://github.com/axetroy/gpm.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/axetroy/gpm.rs/actions/workflows/ci.yml)
![Latest Version](https://img.shields.io/github/v/release/axetroy/gpm.rs.svg)
![License](https://img.shields.io/github/license/axetroy/gpm.rs.svg)
![Repo Size](https://img.shields.io/github/repo-size/axetroy/gpm.rs.svg)

> If you have hundreds of repository, how will you manage them?

This tool helps you manage repository. The directory is hierarchically based on the Git address, similar to Golang's package management, which can organize your hundreds or even thousands of projects.

eg. `https://github.com/axetroy/gpm.rs.git` will be storage at `$ROOT/github.com/axetroy/gpm.rs` just with one command:

```bash
$ gpm clone https://github.com/axetroy/gpm.rs.git
```

### Install

1.  Shell (Mac/Linux)

    ```bash
    curl -fsSL https://github.com/release-lab/install/raw/v1/install.sh | bash -s -- -r=axetroy/gpm.rs -e=gpm
    ```

2.  PowerShell (Windows):

    ```powershell
    $r="axetroy/gpm.rs";$e="gpm";iwr https://github.com/release-lab/install/raw/v1/install.ps1 -useb | iex
    ```

3.  [Github release page](https://github.com/axetroy/gpm.rs/releases)

    Download the executable, then put it in the `$PATH` directory.

## Usage

```sh
# config root path
$ gpm config add root ~/gpm

# clone project instead of git clone
$ gpm clone https://github.com/axetroy/gpm.rs

# clone with git command argument
$ gpm clone https://github.com/axetroy/gpm.rs -- --progress --recursive
```

## Command

| Command                            | Description                        |
| ---------------------------------- | ---------------------------------- |
| gpm clone \<REMOTE\> [OPTIONS]     | Clones repository                  |
| gpm open \<REMOTE\>                | Open repository with file explorer |
| gpm list                           | List repositories                  |
| gpm config                         | Print configure                    |
| gpm config add \<FIELD\> \<VALUE\> | Add configure for a field          |
| gpm config set \<FIELD\> \<VALUE\> | Set configure for a field          |
| gpm config remove \<FIELD\>        | Remove configure for a field       |
| gpm config reset                   | Reset configure                    |

## Relative

- [gpm.js](https://github.com/gpmer/gpm.js) - I wrote with nodejs in many years ago.
- [vscode-gpm](https://github.com/axetroy/vscode-gpm) - Integrate with vscode, I have been using this for a long time.

## LICENSE

The [MIT License](LICENSE)
