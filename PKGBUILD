# Maintainer: edgar1macedosalazar@gmail.com
pkgname=reedit
pkgver=1.0.0
pkgrel=1
pkgdesc="a terminal text editor written in rust"
arch=('x86_64')
url="https://github.com/Edgarmls1/ReEdit"
license=('MIT')
depends=('gcc' 'rust')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cargo build --release
}

package() {
    install -Dm755 "target/release/reedit" "$pkgdir/usr/bin/reedit"
}
