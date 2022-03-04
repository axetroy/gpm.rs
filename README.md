# gpm

[![ci](https://github.com/axetroy/gpm.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/axetroy/gpm.rs/actions/workflows/ci.yml)
![Latest Version](https://img.shields.io/github/v/release/axetroy/gpm.rs.svg)
![License](https://img.shields.io/github/license/axetroy/gpm.rs.svg)
![Repo Size](https://img.shields.io/github/repo-size/axetroy/gpm.rs.svg)

> If you have more than a dozen, even more project in your computer, how will you manage them? This is a problem!

This is what this tool is to be resolved. The directory is hierarchically based on the Git address, similar to Golang's package management, which can organize your hundreds or even thousands of projects.

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

    download the executable file and put the executable file to `$PATH`

## Usage

```sh
# show help information
$ gpm --help
gpm v0.1.0
A cli for manager you project with Golang style

USAGE:
    gpm <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    clone     Clones repos
    config    Update configure
    help      Print this message or the help of the given subcommand(s)

# clone project instead of git clone
$ gpm clone https://github.com/axetroy/gpm.rs
```

## Relative

- [gpm.js](https://github.com/gpmer/gpm.js) - I wrote with nodejs in many years ago.
- [vscode-gpm](https://github.com/axetroy/vscode-gpm) - Integrate with vscode, I have been using this for a long time.

## LICENSE

The [MIT License](LICENSE)
