#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

# https://stackoverflow.com/a/246128
ROOT=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

sudo apt update
sudo apt install ripgrep

if command -v nix &>/dev/null; then
    echo "Skipping nix installation"
else
    bash <(curl -L https://nixos.org/nix/install) --no-daemon
    source ~/.nix-profile/etc/profile.d/nix.sh
fi

if command -v direnv &>/dev/null; then
    echo "Skipping direnv installation"
else
    curl -sfL https://direnv.net/install.sh | sudo -E bin_path=/usr/bin bash
fi

rg -q direnv ~/.bashrc || { echo 'eval "$(direnv hook bash)"' >> ~/.bashrc; }

# NOTE: below is for setting up dotfiles
# if command -v home-manager &>/dev/null; then
#     echo "Skipping home-manager installation"
# else
#     nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
#     nix-channel --update
#     nix-shell '<home-manager>' -A install
# fi
#
# [ -d ".dot" ] || git clone git@github.com:nwtnni/.dot.git
# cd ~/.dot
# sed -i"" "s/local = true/local = false/" home.nix
# home-manager init --switch . -b bak

cd "$ROOT/../data"
[ -f 'email.txt' ] || { wget -O email.txt.gz 'https://www.dropbox.com/scl/fi/fif8lg9vwosftb3hyew61/email.txt.gz?rlkey=5649fx3b4ae8mnqg6e7ts6rrl&st=hpbbt6hx&dl=1' && gunzip email.txt.gz; }
[ -f 'ipv4.bin' ] || { wget -O ipv4.bin.gz 'https://www.dropbox.com/scl/fi/x2jypzq32e9e4stemckw8/ipv4.bin.gz?rlkey=q61foqtcnt5hhx6gc2hmegn39&st=y6k757n9&dl=1' && gunzip ipv4.bin.gz; }
[ -f 'snowflake.bin' ] || { wget -O snowflake.bin.gz 'https://www.dropbox.com/scl/fi/q9vnnitkxj8cu71eer3nj/snowflake.bin.gz?rlkey=zi5f62591w30qbbpl76pk6fc6&st=wk02kan3&dl=1' && gunzip snowflake.bin.gz; }
[ -f 'url.txt' ] || { wget -O url.txt.gz 'https://www.dropbox.com/scl/fi/eurnrj268bdxhbjjzmq7z/url.txt.gz?rlkey=9upb8mmygnjvjgbtlnwya9y73&st=ssm465fb&dl=1' && gunzip url.txt.gz; }

cd "$ROOT/.."
git submodule update --init --recursive
rg -q "flake" .envrc || { echo 'use flake' >> .envrc; }

# https://github.com/direnv/direnv/issues/262
direnv allow .
eval "$(direnv export bash)"

./script/normalize.sh

# Build vendored hdrhistogram submodule
cd "$ROOT/../extern/HdrHistogram_py"
python3 setup.py build

cd "$ROOT/../extern/mimalloc_rust/libmimalloc-sys/c_src/mimalloc/v3/"
mkdir -p build && cd build
cmake .. -DCMAKE_C_FLAGS="-DMI_STAT=2"
make -j mimalloc-static

cd ~
[ -d 'turso-arctic' ] || git clone https://github.com/jennyhour/turso-arctic.git
cd ~/turso-arctic
direnv allow .
eval "$(direnv export bash)"
./perf/throughput/turso/scripts/setup.sh

cd ~
[ -d 'rocksdb-arctic' ] || git clone https://github.com/nwtnni/rocksdb-arctic.git
cd ~/rocksdb-arctic
direnv allow .
eval "$(direnv export bash)"
./setup.sh
