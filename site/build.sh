#!/bin/sh

set -e

cargo test --verbose
cargo install cobalt-bin
cd website
cobalt build
cd ../
cargo install mdbook
mdbook build book
mv website/_site site
mv book/book site/guide

