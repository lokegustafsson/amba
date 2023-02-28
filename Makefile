.PHONY: upload upload-amba upload-s2e

--upload-impl: eurydice-private-key
	nix build '.#$(TARGET)' --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive '.#$(TARGET)'
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy '.#$(TARGET)' --to ssh://nix.u3836.se

upload-amba: TARGET=amba
upload-amba: --upload-impl

upload-s2e: TARGET=s2e
upload-s2e: --upload-impl

upload: upload-amba upload-s2e
