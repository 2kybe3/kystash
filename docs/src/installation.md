# Installation

There are multiple ways to install kystash. Choose any one of the methods below that best suit your needs.

## Using cargo install

This builds kystash from source.

#### Unstable
```bash
cargo install --git https://git.kybe.xyz/2kybe3/kystash
```

#### Stable
TBA

## Using nix

The flake defines a nix cache server you can use if you don't want to build from source.

#### Profile (unstable)
```bash
nix profile install git+https://git.kybe.xyz/2kybe3/kystash
```

#### One time run (unstable)
```bash
nix run git+https://git.kybe.xyz/2kybe3/kystash
```

#### Profile (stable)
TBA

#### One time run (stable)
TBA
