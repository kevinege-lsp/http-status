# http-status

An [Akamai Functions](https://www.akamai.com/products/akamai-functions) (Spin) app written in Rust that responds with whatever HTTP status code you ask for.

## Usage

| Request | Response |
|---|---|
| `/?status=404` | `404 Not Found` |
| `/?status=302` | `302 Found` |
| `/?status=503` | `503 Service Unavailable` |
| `/?status=100` (any 1xx) | `200 OK` with a note that informational statuses can't be sent as a final response |
| `/?status=799`, `/?status=abc` | `200 OK` with a note that the value isn't a valid HTTP status code |
| `/` (no parameter) | `200 OK` with a usage hint |

Any registered 2xx–5xx status code is accepted. Responses are plain text in the form `<code> <reason>`. 204 and 304 responses have an empty body, as HTTP requires.

## Run locally

Requires [Rust](https://rustup.rs) with the `wasm32-wasip2` target and the [Spin CLI](https://spinframework.dev).

```sh
rustup target add wasm32-wasip2
spin build
spin up
curl -i "http://127.0.0.1:3000/?status=418"
```

## Test

```sh
cargo test
```

## Deploy

```sh
spin aka deploy
```
