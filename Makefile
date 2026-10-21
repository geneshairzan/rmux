PREFIX ?= $(HOME)/.local
BINDIR ?= $(PREFIX)/bin
DATADIR ?= $(PREFIX)/share
APPDIR ?= $(DATADIR)/applications
ICONDIR ?= $(DATADIR)/icons/hicolor/scalable/apps

.PHONY: all build run test install-user clean

all: build

build:
	cargo build --release

run:
	cargo run --release

test:
	cargo test

install-user: build
	mkdir -p $(BINDIR) $(APPDIR) $(ICONDIR)
	cp target/release/rmux $(BINDIR)/rmux
	chmod +x $(BINDIR)/rmux
	ln -sf $(BINDIR)/rmux $(BINDIR)/cmux-native
	cp data/rmux.desktop $(APPDIR)/rmux.desktop
	cp data/cmux-native.desktop $(APPDIR)/cmux-native.desktop
	cp data/icons/rmux.svg $(ICONDIR)/rmux.svg
	cp data/icons/cmux-native.svg $(ICONDIR)/cmux-native.svg
	update-desktop-database $(APPDIR) 2>/dev/null || true
	gtk-update-icon-cache $(DATADIR)/icons/hicolor 2>/dev/null || true
	@echo "rmux Rust binary installed successfully to $(PREFIX)!"

clean:
	cargo clean
	rm -rf src_py pyproject.toml setup.py
