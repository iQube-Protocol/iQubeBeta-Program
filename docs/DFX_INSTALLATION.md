# DFX Installation Manual

Since the automated dfx installation failed, please install manually:

## Option 1: Manual Installation

```bash
# Download dfx directly
curl -fsSL https://github.com/dfinity/sdk/releases/download/0.29.1/dfx-x86_64-apple-darwin.tar.gz -o /tmp/dfx.tar.gz

# Extract and install
cd /tmp
tar -xzf dfx.tar.gz
mkdir -p "$HOME/Library/Application Support/org.dfinity.dfx/bin"
mv dfx "$HOME/Library/Application Support/org.dfinity.dfx/bin/"

# Add to PATH
echo 'export PATH="$HOME/Library/Application Support/org.dfinity.dfx/bin:$PATH"' >> ~/.zshenv
source ~/.zshenv

# Verify installation
dfx --version
```

## Option 2: Using Homebrew

```bash
brew install dfinity/tap/dfx
```

## After Installation

1. Start local replica:
```bash
cd /Users/hal1/CascadeProjects/iQubeBeta-Program
dfx start --background
```

2. Deploy canisters:
```bash
dfx deploy
```

3. Test proof-of-state canister:
```bash
dfx canister call proof_of_state submit_receipt '("test_data", "test_metadata")'
```

## Troubleshooting

- If dfx command not found, restart terminal or run `source ~/.zshenv`
- If port conflicts, use `dfx start --background --port 8001`
- For permission issues, ensure dfx binary is executable: `chmod +x "$HOME/Library/Application Support/org.dfinity.dfx/bin/dfx"`
