# Dev

the tauri desktop build is only for dev purpose.

all feature must consider android compatibility

```shell
# start podman for a cloudreve server for dev.
# (blocking, term or killed will stop the container)
scripts/dev_cloudreve.py 
# frontend
pnpm dev
# app
./target/debug/source --debug-cdp-port {CDP_PORT}

```

## Observability

* recommend use playwright-cli skill via `playwright-cli attach --cdp {CDP_PORT}` to interact with the app.
* user launched app will default use 9333 cdp port.

* app data is in local isolated dir :`./tmp/home`. specially,
  db is `./tmp/home/.local/share/top.k88936.source/catalog.sqlite3`

## Test

* NEVER write any new test case (but it is ok to mod existing or write some case to test and clean them when dev)
  unless explicitly required

## Doc

* NEVER write/mod any doc unless explicitly required.

## Others

* NEVER write fallback code unless confirmation
* NEVER write defensive code unless confirmation
* prefer compile time (e.g. type system) solution
* NEVER emoji, full-width punctuation.

* Still in inner and rapid dev, Only consider cloudreve v4 , Never consider migration