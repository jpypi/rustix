RUSTIX=perplexinglabs/rustix:0.1
RUSTIX_DIESEL=perplexinglabs/rustix-diesel:0.1

.PHONY: rustix, migration, update, up, down, stop, start, setup, cleanup

rustix:
	mkdir -p var
	docker build -t $(RUSTIX) -f rustix.Dockerfile .

migration:
	docker build -t $(RUSTIX_DIESEL) -f migration.Dockerfile.

update:
	docker pull registry.gitlab.com/jpypi/rustix/rustix
	POSTGRES_PASSWORD=na docker compose down
	@POSTGRES_PASSWORD=$(shell cat .pw_lock) docker compose up -d

upd: rustix
	@POSTGRES_PASSWORD=$(shell cat .pw_lock) docker compose up

up:
	@POSTGRES_PASSWORD=$(shell cat .pw_lock) docker compose up -d
down:
	POSTGRES_PASSWORD=na docker compose down
stop:
	POSTGRES_PASSWORD=na docker compose stop
start:
	POSTGRES_PASSWORD=na docker compose start

setup:
	@if [ ! -f .pw_lock ]; then\
		head -c 16 /dev/random | base64 > .pw_lock;\
	fi
	POSTGRES_PASSWORD=$(shell cat .pw_lock) docker compose --profile=setup up -d
	mkdir -p var
	docker rm rustix-db-migration-1

cleanup:
	docker image rm $(docker images -f "dangling=true" -q)
