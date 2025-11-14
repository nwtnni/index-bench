#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

cd ~

bash <(curl -L https://nixos.org/nix/install) --no-daemon

curl -sfL https://direnv.net/install.sh | sudo -E bin_path=/usr/bin bash

echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

# git clone git@github.com:nwtnni/.dot.git
# nix-channel --add https://github.com/nix-community/home-manager/archive/master.tar.gz home-manager
# nix-channel --update
# nix-shell '<home-manager>' -A install
# home-manager init --switch .

git clone git@github.com:nwtnni/index-bench.git
cd index-bench
git submodule update --init --recursive
echo "use flake" > .envrc
direnv allow .

cd ~
./index-bench/scripts/normalize.sh
