
all: help

help:

build.steamdeck: SteamDeck/Dockerfile
	cargo fetch
	docker build -t steamdeck-dev SteamDeck
	docker run -it \
		-v "${PWD}":/workspace/app \
		-v "../just-sdl3":/workspace/just-sdl3 \
		steamdeck-dev

build.steamdeck.local:
	JUSTSDL_VENDORED=1 JUSTSDL_LIB_KIND="static" cargo build --bin wavy-mcgee --target=x86_64-unknown-linux-gnu
	cp -v ./target/x86_64-unknown-linux-gnu/debug/wavy-mcgee SteamDeck/
