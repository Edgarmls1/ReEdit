# Maintainer: edgar1macedosalazar@gmail.com
pkgname=reedit
pkgver=1.0
pkgrel=1
pkgdesc="a terminal text editor written in rust"
arch=('x86_64')
url="https://github.com/Edgarmls1/ReEdit"
license=('MIT')
depends=('gcc' 'rust')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/heads/main.tar.gz")
sha256sums=('SKIP')

build() {
    cd "ReEdit-main"
    cargo build --release
}

package() {
    cd "ReEdit-main"
    sudo install -Dm755 "target/release/reedit" "/usr/bin/"
}
