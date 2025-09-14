.PHONY: bootstrap dev test lint format build deploy-dev deploy-stage deploy-prod anchor

bootstrap:
	pnpm i --filter ./apps/** --filter ./packages/**
	cargo build --workspace

lint:
	npx biome check . || true
	cargo clippy --all-targets -- -D warnings || true

format:
	npx biome format .
	cargo fmt --all

dev:
	dfx start --background || true
	dfx deploy
	pnpm -w dev

build:
	pnpm -w build
	cargo build --release --workspace

test:
	pnpm -w test
	cargo test --workspace

anchor:
	dfx canister call proof_of_state anchor '()'

deploy-dev:
	dfx deploy --network local

deploy-stage:
	dfx deploy --network ic

deploy-prod:
	# promote stage artifacts with tag
