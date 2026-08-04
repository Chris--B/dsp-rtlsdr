
.PHONY: all
all: help

.PHONY: help
help:

.PHONY: check
check:
	cargo clippy --all-targets --target=aarch64-apple-darwin
	cargo clippy --all-targets --target=x86_64-unknown-linux-gnu
	cargo clippy --all-targets --target=aarch64-linux-android
	cargo clippy --all-targets --target=x86_64-pc-windows-msvc

.PHONY: build.steamdeck
build.steamdeck: SteamDeck/Dockerfile
	docker build -t steamdeck-dev SteamDeck
	docker run --rm -it \
		-v "${PWD}":/workspace/app \
		-v "../just-sdl3":/workspace/just-sdl3 \
		steamdeck-dev

.PHONY: build.steamdeck.local
build.steamdeck.local:
	JUSTSDL_VENDORED=1 JUSTSDL_LIB_KIND="static" cargo build \
		--bin wavy-mcgee \
		--release \
		--target=x86_64-unknown-linux-gnu
	@echo
	./scripts/build-appimage.py --target=x86_64-unknown-linux-gnu --bin wavy-mcgee --output="SteamDeck/Wavy McGee.AppImage"
	@echo
	@echo "Files in SteamDeck/Wavy McGee.AppImage"
	appimagetool --list "SteamDeck/Wavy McGee.AppImage"

.PHONY: deploy.steamdeck
deploy.steamdeck:
	rsync -avz --progress SteamDeck/ deck@lil-titan:~/rtlsdr/

.PHONY: build.android
build.android:
	JUSTSDL_VENDORED=1 JUSTSDL_LIB_KIND="static" cargo build \
		--bin wavy-mcgee \
		--release \
		--target=aarch64-linux-android
