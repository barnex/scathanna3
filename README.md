# Scathanna 3

This is a 3D engine and game written in [Rust](https://rust-lang.org) from scratch. I.e. without using an existing engine.

## Installation

Tested on Linux, Mac and Windows.

Make sure you have Rust installed from [rustup.rs](http://rustup.rs). Then run:

```
git clone https://github.com/barnex/scathanna3
cd scathanna3

cargo run --release --bin server &
cargo run --release
```

Linux only: install dependencies first(example for Ubuntu):

```
sudo apt install \
	build-essential \
	pkg-config \
	cmake \
	libasound2-dev \
	libfontconfig-dev \
```

## Settings

Edit `settings.toml` for game client settings (controls, server address, player name, ...), and `server.toml` for server settings.




## Gallery


![fig](shots/088-ferris_cove.jpg)
