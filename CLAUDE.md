# RestedPi

A Rust-based IoT automation server for Raspberry Pi. Provides a GraphQL API for managing sensors and switches, executing automations based on boolean expressions, and exposing Prometheus metrics.

## Project Overview

- **Purpose**: Middleware for IoT device management on Raspberry Pi
- **Hardware**: Communicates with I2C devices (MCP9808 temperature, BMP085 pressure, MCP23017 GPIO expander)
- **Storage**: SQLite database for device configuration and state
- **API**: GraphQL endpoint with JWT-based authentication
- **Automation**: Boolean expression engine for automated control logic

## Building & Running

```bash
# Build (add --features raspberrypi for actual hardware support)
cargo build [--features raspberrypi]

# Run server
./restedpi server

# Interactive boolean expression REPL
./restedpi boolean-repl

# Add user credentials
./restedpi add-user --username <name> --password <password>
```

**Configuration**: TOML file at `~/.config/restedpi/config.toml` or `/etc/restedpi/config.toml`

## Architecture

- **Actor Model**: Async message passing via tokio channels
- **Layered Design**: Web (Warp) → Application (Channel Actor) → Hardware (RPI) → Persistence (Diesel/SQLite)
- **Feature Gating**: `raspberrypi` feature enables actual hardware; mock implementations otherwise

## Key Modules

| Module | Purpose |
|--------|---------|
| `app/` | Core state machine, device/input/output abstractions, database ORM |
| `rpi/` | Hardware layer - I2C bus, GPIO, device drivers |
| `config/` | TOML parsing, boolean expression parser (lrlex/lrpar) |
| `graphql/` | Juniper-based GraphQL schema (queries/mutations) |
| `auth/` | Argon2 password hashing, HMAC-SHA256 tokens |
| `webapp/` | Warp routes, static file serving |

---

### Defensive Programming

Core principle: Make implicit invariants explicit and compiler-checked.

**Type Safety**
- **Pattern matching over indexing**: Use slice patterns (`match v.as_slice() { [] => ..., [x] => ..., _ => ... }`) instead of direct indexing
- **Explicit field initialization**: Avoid `..Default::default()` - destructure defaults first if needed
- **Exhaustive pattern matching**: Spell out all enum variants; avoid catch-all `_` patterns
- **Use `TryFrom` for fallible conversions**: Resist `From` for anything that can fail
- **Enums over booleans**: Replace `fn foo(enable: bool, validate: bool)` with descriptive enums
- **Private fields for validation**: Use `_private: ()` field to force constructor usage

**Compiler Assistance**
- **Apply `#[must_use]`** on important return types (Result, Config, builders)
- **Destructure in trait impls** to catch field additions: `let Self { field1, field2 } = self;`
- **Named destructuring over wildcards**: Use `Foo { bar: _, .. }` not `Foo { _, .. }`

**Code Hygiene**
- Always clean all warnings and errors including clippy output
- Use `unreachable!()` for mathematically impossible branches (e.g., `n % 8` matching 8+)

### Recommended Clippy Lints

Consider enabling in `lib.rs`:
```rust
#![warn(clippy::indexing_slicing)]      // Prevent direct slice indexing
#![warn(clippy::wildcard_enum_match_arm)] // Disallow catch-all patterns
#![warn(clippy::must_use_candidate)]    // Suggest #[must_use]
```