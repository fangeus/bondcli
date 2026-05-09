.PHONY: build release install install-prebuilt uninstall clean test lint

VERSION ?= 1.0.0
PREFIX ?= /usr/local/bin

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

lint:
	cargo clippy -- -D warnings
	cargo fmt --check

install: release
	install -m 755 target/release/bondcli $(PREFIX)/bondcli

install-prebuilt:
	@ARCH=$$(uname -m); \
	case $$ARCH in \
		x86_64) FILE="bondcli-linux-x86_64" ;; \
		aarch64) FILE="bondcli-linux-aarch64" ;; \
		*) echo "[ERROR] Unsupported architecture: $$ARCH"; exit 1 ;; \
	esac; \
	curl -L -o /tmp/bondcli "https://github.com/fangeus/bondcli/releases/latest/download/$$FILE"; \
	install -m 755 /tmp/bondcli $(PREFIX)/bondcli; \
	rm -f /tmp/bondcli; \
	echo "[SUCCESS] bondcli installed"

uninstall:
	rm -f $(PREFIX)/bondcli

clean:
	cargo clean
