RUSTIX=perplexinglabs/rustix:0.1

.PHONY: rustix, migration, up, down, stop, start, setup

rustix:
	mkdir -p var
	docker build -t $(RUSTIX) -f Dockerfile .

migration:
	docker build -t perplexinglabs/rustix-diesel:0.1 -f DockerfileMigration .

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
	POSTGRES_PASSWORD=$(shell cat .pw_lock) docker-compose --profile=setup up -d
	mkdir -p var
	docker cp var/. rustix-rustix-1:/usr/share/rustix/
	docker rm rustix-db-migration-1
