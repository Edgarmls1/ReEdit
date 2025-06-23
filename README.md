# ReEdit - Terminal Text Editor

A simple terminal based vim/neovim like text editor

## Dependencies
- rust

## Instalation

### Arch Based Distros

``` bash
git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
makepkg -si
```

### Other Linux Distros and Mac

```bash
git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
cargo build --release
sudo cp target/release/reedit /usr/bin/
```

### Windows

```bash
git clone https://github.com/Edgarmls1/ReEdit.git
cd ReEdit
cargo build --release
Copy-Item target\release\reedit.exe C:\Users\<your_user>\.cargo\bin\
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
