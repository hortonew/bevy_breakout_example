APK_NAME := $(shell grep 'apk_name' Cargo.toml | awk -F '"' '{print $$2}')
PKG_NAME := $(shell grep '^name = ' Cargo.toml | awk -F '"' '{print $$2}' | head -n 1)

setup:
	cargo install xbuild
	cargo install cargo-apk
	brew install --cask adoptopenjdk
	brew install kotlin gradle llvm squashfs ideviceinstaller
	rustup target add aarch64-linux-android armv7-linux-androideabi
	x doctor

# export ANDROID_HOME=/Users/$USER/Library/Android/sdk
# export NDK_HOME=$ANDROID_HOME/ndk/26.2.11394342
# export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
# export PATH=$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH

run:
	cargo run

# cargo build --target aarch64-linux-android
build:
	cargo build

run_on_android:
	cargo apk run -p $(APK_NAME) --lib

apk_release_debug:
	x build --release --platform android --store play

apk_release:
	cargo apk build -p $(APK_NAME) --release --lib

wasm_release:
	cargo build --release --target wasm32-unknown-unknown
	rm -rf ./webbuild/out/
	rm -rf ./webbuild/assets/
	wasm-bindgen --out-dir ./webbuild/out --target web ./target/wasm32-unknown-unknown/release/$(PKG_NAME).wasm
	cp -r assets ./webbuild/
	rm -f webbuild.zip
	zip -r webbuild.zip webbuild
