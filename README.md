# nix-eval-cache

Status: alpha

skips build/evaluation based on modification date of nix files.

Usage:

```console
$ cat > foo.nix <<EOF
with import <nixpkgs> {};
pkgs.hello
EOF
$ cargo build
$ time ./target/debug/nix-eval-cache result nix-build foo.nix
/nix/store/g124820p9hlv4lj8qplzxw1c44dxaw1k-hello-2.12
./target/debug/nix-eval-cache result nix-build foo.nix  0,42s user 0,12s system 82% cpu 0,657 total
$ time ./target/debug/nix-eval-cache result nix-build foo.nix
skip build
./target/debug/nix-eval-cache result nix-build foo.nix  0,00s user 0,01s system 91% cpu 0,009 total
```
