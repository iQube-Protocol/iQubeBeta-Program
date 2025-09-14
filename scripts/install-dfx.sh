#!/bin/bash

# Install dfx manually without interactive prompts
echo "Installing dfx for ICP development..."

# Create dfx directory
mkdir -p "$HOME/Library/Application Support/org.dfinity.dfx/bin"

# Download and install dfx (latest version)
PLATFORM="x86_64-apple-darwin"
TARBALL="dfx-${PLATFORM}.tar.gz"
URL="https://github.com/dfinity/sdk/releases/download/0.29.1/dfx-x86_64-apple-darwin.tar.gz"

cd /tmp
curl -fsSL "$URL" -o "$TARBALL"
tar -xzf "$TARBALL"
mv dfx "$HOME/Library/Application Support/org.dfinity.dfx/bin/"

# Add to PATH
echo 'export PATH="$HOME/Library/Application Support/org.dfinity.dfx/bin:$PATH"' >> ~/.zshenv

# Source the new PATH
export PATH="$HOME/Library/Application Support/org.dfinity.dfx/bin:$PATH"

echo "dfx installed successfully!"
dfx --version
