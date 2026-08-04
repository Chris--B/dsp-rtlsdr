
.PHONY: all
all: help

.PHONY: help
help:

.PHONY: build.steamdeck
build.steamdeck: SteamDeck/Dockerfile
	cargo fetch
	docker build -t steamdeck-dev SteamDeck
	docker run -it \
		-v "${PWD}":/workspace/app \
		-v "../just-sdl3":/workspace/just-sdl3 \
		steamdeck-dev

.PHONY: build.steamdeck.local
build.steamdeck.local:
	JUSTSDL_VENDORED=1 JUSTSDL_LIB_KIND="static" cargo build \
		--bin wavy-mcgee \
		--release \
		--target=x86_64-unknown-linux-gnu
	./package-appimage.sh

.PHONY: deploy.steamdeck
deploy.steamdeck:
	rsync -avz --progress SteamDeck/ deck@lil-titan:~/rtlsdr/
