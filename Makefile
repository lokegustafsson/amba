upload: eurydice-private-key
	nix build .#amba --accept-flake-config
	nix build .#s2e --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive ./result-bin
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy .#amba --to ssh://nix.u3836.se
	NIX_SSHOPTS="-p1234" nix copy .#s2e --to ssh://nix.u3836.se

.PHONY: clean
