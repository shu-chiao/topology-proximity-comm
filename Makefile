# topology-proximity-comm — common workflows

ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
COMPOSE := docker compose -f infra/docker-compose.yml

.PHONY: default help docker-up docker-down check-bridge install-zenoh-bridge install-plugin build-sample3

default: help

help:
	@echo "Targets:"
	@echo "  make docker-up | docker-down — local zenohd (infra/docker-compose.yml)"
	@echo "  make check-bridge            — zenoh-bridge-ros2dds path/version"
	@echo "  make install-zenoh-bridge    — APT: zenoh-bridge-ros2dds (sudo)"
	@echo "  make install-plugin          — APT: zenoh-plugin-ros2dds for zenohd"
	@echo "  make build-sample3           — cargo build in sample 3 rust/"

docker-up:
	$(COMPOSE) up -d

docker-down:
	$(COMPOSE) down

check-bridge:
	@command -v zenoh-bridge-ros2dds >/dev/null 2>&1 && zenoh-bridge-ros2dds --version \
		|| (echo 'zenoh-bridge-ros2dds not on PATH — run make install-zenoh-bridge' >&2; exit 1)

build-sample3:
	cargo build --manifest-path samples/03-dds-zenoh-bridge/rust/Cargo.toml

install-zenoh-bridge:
	@echo "Installing Eclipse Zenoh apt repo + zenoh-bridge-ros2dds …"
	curl -fsSL https://download.eclipse.org/zenoh/debian-repo/zenoh-public-key \
		| sudo gpg --dearmor --yes -o /etc/apt/keyrings/zenoh-public-key.gpg
	echo 'deb [signed-by=/etc/apt/keyrings/zenoh-public-key.gpg] https://download.eclipse.org/zenoh/debian-repo/ /' \
		| sudo tee /etc/apt/sources.list.d/zenoh.list >/dev/null
	sudo apt-get update -y
	sudo DEBIAN_FRONTEND=noninteractive apt-get install -y zenoh-bridge-ros2dds
	@echo "Installed:" && zenoh-bridge-ros2dds --version

install-plugin:
	@echo "Installing zenoh-plugin-ros2dds (.so for zenohd only) …"
	curl -fsSL https://download.eclipse.org/zenoh/debian-repo/zenoh-public-key \
		| sudo gpg --dearmor --yes -o /etc/apt/keyrings/zenoh-public-key.gpg
	echo 'deb [signed-by=/etc/apt/keyrings/zenoh-public-key.gpg] https://download.eclipse.org/zenoh/debian-repo/ /' \
		| sudo tee /etc/apt/sources.list.d/zenoh.list >/dev/null
	sudo apt-get update -y
	sudo DEBIAN_FRONTEND=noninteractive apt-get install -y zenoh-plugin-ros2dds
