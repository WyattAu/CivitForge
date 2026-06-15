# CivitForge — Local Development Makefile
# Usage: make [target]

.PHONY: build run test fmt clippy smoke clean compose-up compose-down migrate hooks

CARGO  ?= cargo
DATABASE_URL ?= postgres://civit:civit@localhost:5432/civit
REDIS_URL    ?= redis://localhost:6379
JWT_SECRET   ?= change-me-change-me-dev-secret
HOST         ?= 127.0.0.1
PORT         ?= 9091
STORAGE      ?= /tmp/civit-repos
RUST_LOG     ?= info
BIN          ?= target/release/civit-core

# ── Build ──────────────────────────────────────────────────────
build:
	$(CARGO) build --release -p civit-core

build-debug:
	$(CARGO) build -p civit-core

# ── Run (requires docker compose up first) ─────────────────────
run: build
	DATABASE_URL=$(DATABASE_URL) \
	REDIS_URL=$(REDIS_URL) \
	JWT_SECRET=$(JWT_SECRET) \
	CIVIT_HOST=$(HOST) \
	CIVIT_PORT=$(PORT) \
	RUST_LOG=$(RUST_LOG) \
	CIVIT_STORAGE_PATH=$(STORAGE) \
	$(BIN)

run-debug: build-debug
	DATABASE_URL=$(DATABASE_URL) \
	REDIS_URL=$(REDIS_URL) \
	JWT_SECRET=$(JWT_SECRET) \
	CIVIT_HOST=$(HOST) \
	CIVIT_PORT=$(PORT) \
	RUST_LOG=debug \
	CIVIT_STORAGE_PATH=$(STORAGE) \
	./target/debug/civit-core

# ── Test ───────────────────────────────────────────────────────
test:
	$(CARGO) test --workspace --locked

test-core:
	$(CARGO) test -p civit-core --locked

# ── Lint ──────────────────────────────────────────────────────
fmt:
	$(CARGO) fmt --check --all

fmt-fix:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

lint: fmt clippy

# ── Smoke Test (requires server running) ───────────────────────
smoke:
	bash smoke-test.sh

# ── Docker Compose ────────────────────────────────────────────
compose-up:
	docker compose up -d

compose-down:
	docker compose down

# ── Clean ──────────────────────────────────────────────────────
clean:
	$(CARGO) clean
	rm -rf /tmp/civit-repos /tmp/civit-server.log /tmp/civit-smoke-body

# ── Pre-commit hooks ───────────────────────────────────────────
hooks:
	git config core.hooksPath .githooks
	@echo "Pre-commit hooks activated (.githooks/pre-commit)"
	@echo "Bypass: SKIP_PRE_COMMIT=1 git commit ..."

# ── Full cycle: compose → build → test → lint → smoke ─────────
ci-local: compose-up build test lint smoke
