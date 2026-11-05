.PHONY: all build check test parity fmt docker-build docker-run docker-stop audit-site oracle help

CARGO ?= cargo
UPSTREAM_TESTS ?= $(firstword $(wildcard $(VNU_UPSTREAM_TESTS) ../validator-upstream/tests ../../validator-upstream/tests ./validator-upstream/tests))
DOCKER_IMAGE ?= vnu-rs:latest
DOCKER_PORT ?= 8889

all: check test

help:
	@echo "vnu-rs Development & Testing Commands:"
	@echo "  make build         - Build vnu-rs release binary"
	@echo "  make test          - Run internal unit & integration tests"
	@echo "  make parity        - Run 100% upstream W3C test suite (5,824 tests)"
	@echo "  make check         - Type check all crates"
	@echo "  make fmt           - Format codebase with rustfmt"
	@echo "  make docker-build  - Build minimal Debian release Docker image"
	@echo "  make docker-run    - Run vnu-rs daemon in Docker (:$(DOCKER_PORT)->8888)"
	@echo "  make docker-stop   - Stop and remove vnu-rs Docker container"
	@echo "  make oracle        - Run differential oracle against Java vnu (:8888)"
	@echo "  make audit-site    - Crawl and compare production pages against Java & Rust"

build:
	$(CARGO) build --release

check:
	$(CARGO) check --workspace

test:
	$(CARGO) test --workspace

parity:
	@if [ -z "$(UPSTREAM_TESTS)" ] || [ ! -d "$(UPSTREAM_TESTS)" ]; then \
		echo "Error: Upstream tests directory not found. Please set VNU_UPSTREAM_TESTS=/path/to/validator-upstream/tests"; \
		exit 1; \
	fi
	VNU_UPSTREAM_TESTS=$(UPSTREAM_TESTS) $(CARGO) run --release --bin eval_parity

fmt:
	$(CARGO) fmt --all

docker-build:
	docker build -t $(DOCKER_IMAGE) .

docker-run:
	docker rm -f vnu-rs 2>/dev/null || true
	docker run -d --name vnu-rs -p 127.0.0.1:$(DOCKER_PORT):8888 -e VNU_BIND_ADDR=0.0.0.0 $(DOCKER_IMAGE)
	@echo "vnu-rs running at http://127.0.0.1:$(DOCKER_PORT)/?out=json"

docker-stop:
	docker rm -f vnu-rs 2>/dev/null || true

oracle:
	$(CARGO) test -p vnu-oracle -- --nocapture

audit-site:
	@SITEMAP_URL="$(SITEMAP)" python3 ./scripts/audit_site.py
