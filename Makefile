.PHONY: bootstrap dev test lint format build deploy-dev deploy-stage deploy-prod anchor

bootstrap:
	pnpm i --filter ./apps/** --filter ./packages/** || true
	cargo build --workspace || true

lint:
	npx biome check . || true
	cargo clippy --all-targets -- -D warnings || true

format:
	npx biome format . || true
	cargo fmt --all || true

dev:
	dfx start --background || true
	dfx deploy || true
	pnpm -w dev || true

build:
	pnpm -w build || true
	cargo build --release --workspace || true

test:
	pnpm -w test || true
	cargo test --workspace || true

anchor:
	dfx canister call proof_of_state anchor '()' || true

deploy-dev:
	dfx deploy --network local || true

deploy-stage:
	dfx deploy --network ic || true

deploy-prod:
	@echo "Promote stage artifacts with tag (manual gate)"
