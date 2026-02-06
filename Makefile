.PHONY: build release clean github-release

BINARY_NAME=csep

RELEASE_DIR=./release

PLATFORM=$(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH ?= $(shell uname -m | tr '[:upper:]' '[:lower:]')

build:
	@echo "Building the application..."
	@if [ "$(TARGET)" = "" ]; then \
		cargo build --release; \
	else \
		cargo build --release --target $(TARGET); \
	fi

clean:
	@echo "Cleaning up..."
	@cargo clean
	@rm -rf $(RELEASE_DIR)

tarball: build
	@echo "Packaging the release..."
	@mkdir -p $(RELEASE_DIR)
	@if [ "$(TARGET)" = "" ]; then \
		tar -czf $(RELEASE_DIR)/$(BINARY_NAME)-$(PLATFORM)-$(ARCH).tar.gz -C target/release $(BINARY_NAME); \
	else \
		tar -czf $(RELEASE_DIR)/$(BINARY_NAME)-$(PLATFORM)-$(ARCH).tar.gz -C target/$(TARGET)/release $(BINARY_NAME); \
	fi
	@echo "Release package created: $(RELEASE_DIR)/$(BINARY_NAME)-$(PLATFORM)-$(ARCH).tar.gz"


tarball-publish: tarball
	@TARBALL=$(BINARY_NAME)-$(PLATFORM)-$(ARCH).tar.gz; \
	echo "Sending tarball $$TARBALL to script"; \
	sh ./scripts/publish_asset.sh $$TARBALL

homebrew:
	@sh ./scripts/homebrew.sh
