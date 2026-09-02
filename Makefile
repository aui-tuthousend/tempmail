.PHONY: dev redis-up redis-down api smtp processor cleanup frontend check

SHELL := /bin/bash

COMPOSE := docker compose -f docker-compose.yml
FRONTEND_DIR := frontend

redis-up:
	$(COMPOSE) up -d dragonfly

redis-down:
	$(COMPOSE) down

api:
	cargo run -p api-server

smtp:
	cargo run -p smtp-receiver

processor:
	cargo run -p email-processor

cleanup:
	cargo run -p cleanup-worker

frontend:
	cd $(FRONTEND_DIR) && bun run dev

dev: redis-up
	trap 'kill 0' INT TERM EXIT; \
	cargo run -p api-server & \
	cargo run -p smtp-receiver & \
	cargo run -p email-processor & \
	cargo run -p cleanup-worker & \
	cd $(FRONTEND_DIR) && bun run dev & \
	wait

check:
	cargo check --workspace
	cd $(FRONTEND_DIR) && bun run typecheck
