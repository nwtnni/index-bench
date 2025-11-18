#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

cd ~

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

rg -q direnv ~/.bashrc || echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

if command -v home-manager &>/dev/null; then
    echo "Skipping home-manager installation"
else
    nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
    nix-channel --update
    nix-shell '<home-manager>' -A install
fi

[ -d ".dot" ] || git clone git@github.com:nwtnni/.dot.git
cd ~/.dot
sed -i"" "s/local = true/local = false/" home.nix
home-manager init --switch . -b bak

cd ~
[ -d "index-bench" ] || git clone git@github.com:nwtnni/index-bench.git
cd ~/index-bench
git submodule update --init --recursive
rg -q "flake" .envrc || echo 'use flake' >> .envrc
direnv allow .
./script/normalize.sh
