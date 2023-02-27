upload:
	nix build . --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive ./result-bin
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy . --to ssh://nix.u3836.se

.PHONY: clean
