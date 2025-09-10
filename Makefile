include .env
export $(shell sed 's/=.*//' .env)

dev:
	@cargo install --locked watchexec-cli
	@watchexec -e rs -r cargo run

prod:
	@docker compose up -d api

start-db:
	@docker compose up -d $(DATABASE_TYPE) 

stop-db:
	@docker compose stop $(DATABASE_TYPE)

test:
	@cargo test -- --test-threads=1

