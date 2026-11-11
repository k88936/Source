# Dev

* the tauri desktop build is only for dev purpose.

* all feature must consider android compatibility


## Observability

### full app with cdp compatible port
```shell
# start podman for a cloudreve server for dev.
# (blocking, term or killed will stop the container)
scripts/dev_cloudreve.py 
# frontend
pnpm dev
# app (run in a virtual display to avoid disturbing user)
# user launched app will default use 9333 cdp port, yours should avoid it.
# you can restart user launched app by restart RustRover's run config `app` (for example to apply rust side change)

# app data is in local isolated dir :`./tmp/home`. specially,
# db is `./tmp/home/.local/share/top.k88936.source/catalog.sqlite3`
xvfb-run -a -s "-screen 0 430x932x24" ./target/debug/source --debug-cdp-port {CDP_PORT}

playwright attach --cdp {CDP_PORT}
```

### frontend-only with mocked Tauri call
```shell
pnpm demo:dev

playwright open http://localhost:5174/Source/app/sync
```

* NEVER use android vm to debug or test unless required.

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

* ALWAYS IN ZH
