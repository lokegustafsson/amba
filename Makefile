.PHONY: upload upload-amba upload-s2e get-libamba-symbols

upload: upload-amba upload-s2e upload-libamba

upload-amba: eurydice-private-key
	nix build '.#amba' --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive ./result-bin
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy '.#amba' --to ssh://nix.u3836.se

upload-s2e: eurydice-private-key
	nix build '.#s2e' --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive ./result
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy '.#s2e' --to ssh://nix.u3836.se

upload-libamba: eurydice-private-key
	nix build '.#libamba' --accept-flake-config
	doas nix store sign --key-file ./eurydice-private-key --recursive ./result
	nix store verify --trusted-public-keys $$(nix key convert-secret-to-public < ./eurydice-private-key) .
	NIX_SSHOPTS="-p1234" nix copy '.#libamba' --to ssh://nix.u3836.se

get-libamba-symbols:
	nix build '.#libamba'
	nm -D $$(nix path-info '.#libamba')/lib/libamba.so \
		| rg ' U ([^\s]+)' -or '$$1' \
		| rg -v 'GLIBC|CXXABI|GCC' \
		| c++filt \
		| printf "\nDynamic symbols required by libamba:\n\n$$(cat -)"

compile_flags.txt:
	make -C crates/libamba ../../compile_flags.txt
