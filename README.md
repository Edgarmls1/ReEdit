# ReEdit - Terminal Text Editor

A simple terminal based vim/neovim like text editor

## Dependencies
- rust
- nerd font (optional)

## Instalation

### Arch Based Distros

``` bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo pacman -S git

git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
makepkg -si
```

### Other Linux Distros and Mac

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo apt install git # for debian based
sudo dnf install git # for redhat based
brew install git # for macos

git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
cargo build --release
sudo cp target/release/reedit /usr/bin/
```

### Windows

```bash
winget install -e --id Git.Git

git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
cargo build --release
cargo install --path .
```

## Usage

```bash
reedit -h or reedit --help
```

## Features

- [x] insert and command mode
- [x] file browser (sidebar)
- [x] line numbers
- [ ] customization
- [ ] LSP
