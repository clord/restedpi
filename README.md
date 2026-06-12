# RestedPi

IoT automation middleware for the Raspberry Pi. RestedPi talks to I2C sensors
and GPIO expanders, runs automations written in a small boolean-expression
language, and exposes everything through a GraphQL API with Prometheus-style
metrics.

## Features

- **Hardware support** over I2C:
  - MCP9808 high-accuracy temperature sensor
  - BMP085 pressure/temperature sensor
  - MCP23017 16-port GPIO expander (inputs and outputs)
- **Automations**: attach a boolean expression to any output and the server
  recomputes it as state changes (e.g. turn on a fan when a temperature
  reading crosses a threshold, or switch lights relative to sunrise/sunset).
- **GraphQL API** with JWT-style token authentication for managing devices,
  inputs, and outputs at runtime.
- **Metrics** endpoint exposing current input/output values in Prometheus
  text format.
- **SQLite storage** for device configuration and state.

## Building

```bash
# Host build with mock hardware (the default; no Raspberry Pi required)
cargo build

# Build with real Raspberry Pi hardware support (rppal-based I2C/GPIO)
cargo build --features raspberrypi

# Run the test suite (mock GPIO implementations)
cargo test --features mock-gpio
```

## Running

```bash
# Start the server
restedpi server

# Interactive REPL for trying out boolean expressions
restedpi boolean-repl

# Add a user to the config file (prompts for password if not given)
restedpi add-user --username <name> [--password <password>]
```

## Configuration

RestedPi reads a TOML config file from `~/.config/restedpi/config.toml` or
`/etc/restedpi/config.toml` (override with `--config-file <path>`).

```toml
# Example config.toml
listen = "0.0.0.0"
port = 3030
i2cbus = 1

# Location used by the "here" value in expressions (sunrise/sunset math)
lat = 49.2
long = -123.1

# Directory containing the SQLite database (rpi.sql3)
db_path = "/var/lib/restedpi"

# File containing the session-signing secret (hex-encoded random bytes).
# If unset, the APP_SECRET environment variable is used instead.
app_secret_path = "/etc/restedpi/secret"

# Optional TLS; provide both to serve HTTPS
tls_key_path = "/etc/restedpi/key.pem"
tls_cert_path = "/etc/restedpi/cert.pem"
```

Every setting can also be supplied as an environment variable with the
`RESTEDPI_` prefix, which takes priority over the config file:

- `RESTEDPI_PORT=8080`
- `RESTEDPI_LISTEN=0.0.0.0`
- `RESTEDPI_I2CBUS=1`
- `RESTEDPI_LAT=45.5`
- `RESTEDPI_LONG=-122.6`
- `RESTEDPI_DB_PATH=/var/lib/restedpi`
- `RESTEDPI_APP_SECRET_PATH=/etc/restedpi/secret`
- `RESTEDPI_TLS_KEY_PATH=/etc/restedpi/key.pem`
- `RESTEDPI_TLS_CERT_PATH=/etc/restedpi/cert.pem`

## Endpoints

| Path | Description |
|------|-------------|
| `/graphql` | GraphQL queries and mutations (POST); GraphQL Playground (GET) |
| `/subscriptions` | GraphQL subscriptions over WebSocket |
| `/metrics` | Current input/output values in Prometheus text format |
| `/` | Single-page web app |

## Boolean expression language

Automations and the REPL use a small expression language. An expression
evaluates to true or false; outputs with an automation script are set to the
result whenever state changes.

```text
# Logic
true
a and b or not c
(door_open or window_open) and not alarm_armed

# Comparisons on values: ==, !=, <, <=, >, >=, between
read(greenhouse, degC) > 21.5
read(pressure_sensor, kpa) between 95 and 105

# Approximate equality
plus/minus 0.5, read(greenhouse, degC) == 21

# Time and sun position
hour_of_day(now) between 7 and 10
hour_of_day(now) > hour_of_sunrise(here, now)
week_day(now) < 5

# Arithmetic and interpolation
read(indoor, degC) - read(outdoor, degC) > 5
lerp(1, 0.5, 2) == 1.5
```

Building blocks:

- `read(name, unit)` reads an input by name; units are `degC`, `kpa`, `bool`
- `hour_of_day(now)`, `minute_of_hour(now)`, `week_day(now)`,
  `day_of_year(now)`, `month_of_year(now)`, `year(now)`
- `hour_of_sunrise(here, now)`, `hour_of_sunset(here, now)`,
  `hours_of_daylight(here, now)` — `here` comes from the configured lat/long,
  or write an explicit location like `49.2 degN 123.1 degW`
- `lerp(a, t, b)` linearly interpolates between `a` and `b`
- Arithmetic: `+`, `-`, `*`, `/`, parentheses

Try expressions interactively with `restedpi boolean-repl`.

## License

MIT — see [LICENSE](LICENSE).
