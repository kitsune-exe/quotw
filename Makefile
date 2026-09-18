.PHONY: build release install uninstall clean help

BINARY_NAME = ratatui-quote

build:
	cargo build

release:
	cargo build --release

install:
	cargo install --path .
	cargo clean
	@echo "Installed $(BINARY_NAME) via cargo install and cleaned artifacts"

uninstall:
	cargo uninstall $(BINARY_NAME)
	@echo "Uninstalled $(BINARY_NAME) via cargo uninstall"

clean:
	cargo clean
	@echo "Cleaned build artifacts"

help:
	@echo "Available targets:"
	@echo "  build      - Build debug version"
	@echo "  release    - Build release version"
	@echo "  install    - Install via cargo install"
	@echo "  uninstall  - Uninstall via cargo uninstall"
	@echo "  clean      - Remove all build artifacts"
	@echo "  help       - Show this help"

.DEFAULT_GOAL := install