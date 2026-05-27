
check: clear fmt clippy

fmt:
    @echo "==> Formatting..."
    @cargo fmt --all
    @cargo fix --all-targets --allow-dirty


clippy:
    @echo "==> Linting with Clippy..."
    @cargo clippy --all-targets --fix -- -D warnings # Fix what can be fixed...
    @cargo clippy --all-targets -- -D warnings

clear:
    clear

test:
    @echo "==> Running tests..."
    @cargo test -- --nocapture

ex-local:
    cargo run --example local_candle_example --features local-candle --release

ex-bedrock:
    cargo run --example bedrock_example --features aws-bedrock --release

ex-opencode:
    cargo run --example opencode_go_example --features http-client --release
