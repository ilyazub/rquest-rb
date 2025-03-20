require "mkmf"

# Check for Rust toolchain
if !system("which cargo > /dev/null 2>&1")
  raise "Rust toolchain not found. Please install Rust: https://rustup.rs/"
end

# Create Makefile
create_makefile("rquest_rb") 