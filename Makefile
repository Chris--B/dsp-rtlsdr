
.PHONY: all
all: help

.PHONY: help
help:

.PHONY: clean
clean:
	cargo clean

.PHONY: check
check:
	JUSTSDL_NO_LINK=1 cargo clippy --all-targets --target=aarch64-apple-darwin
	JUSTSDL_NO_LINK=1 cargo clippy --all-targets --target=x86_64-unknown-linux-gnu
	JUSTSDL_NO_LINK=1 cargo clippy --all-targets --target=aarch64-linux-android
	JUSTSDL_NO_LINK=1 cargo clippy --all-targets --target=x86_64-pc-windows-msvc

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

ANDROID_TOOLCHAIN     := $(shell find "$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/" -maxdepth 2 -name bin)
ANDROID_PLATFORM_ROOT := $(shell find "$(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/" -maxdepth 2 -name sysroot)
.PHONY: build.android
build.android:
	@echo "Using Android NDK at ANDROID_NDK_HOME=${ANDROID_NDK_HOME}"
	env \
	"CMAKE_TOOLCHAIN_FILE_aarch64-linux-android"=${ANDROID_NDK_HOME}/build/cmake/android.toolchain.cmake     \
	"CC_aarch64_linux_android"="${ANDROID_TOOLCHAIN}/aarch64-linux-android31-clang"                          \
	"CXX_aarch64_linux_android"="${ANDROID_TOOLCHAIN}/aarch64-linux-android31-clang++"                       \
	"AR_aarch64_linux_android"="${ANDROID_TOOLCHAIN}/llvm-ar"                                                \
	"CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"="${ANDROID_TOOLCHAIN}/aarch64-linux-android31-clang"         \
	"SDL_ANDROID_PLATFORM_ROOT"="${ANDROID_PLATFORM_ROOT}"                                                   \
	"SDL_ANDROID_PLATFORM_ANDROID_JAR"="${ANDROID_SDK_ROOT}/platforms/android-31/android.jar"                \
	"ANDROID_ABI"=arm64-v8a                                                                                  \
	"ANDROID_PLATFORM"=android-31                                                                            \
	"TARGET_PKG_CONFIG_ALLOW_CROSS"=1 \
	TARGET_PKG_CONFIG_SYSROOT_DIR="target/android_sysroot" \
	"JUSTSDL_VENDORED"=1                                                                                     \
	"JUSTSDL_LIB_KIND"="static"                                                                              \
	cargo build                                                                                              \
	    --bin wavy-mcgee                                                                                     \
	    --release                                                                                            \
	    --target=aarch64-linux-android                                                                       
