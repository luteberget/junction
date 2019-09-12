# This script takes care of testing your crate

set -ex

cd lib/
git clone https://github.com/luteberget/lsqr-rs.git
git clone https://github.com/luteberget/railplot.git
git clone https://github.com/koengit/trainspotting.git
cd ..

cargo test --target $TARGET
cargo build --target $TARGET --release
