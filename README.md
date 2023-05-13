# logdog
Log indexing and search utilities. Licenced under MIT Licence

## Features

The module aim to be able to index and search logs at production scale.

The aim of this project is to be able to manage over 1 billion logs on a single node instance, providing reasonable ingestion and query time.
This have been tested so far on over 100M logs with very good response and query times.

Several utilities are bundled:

- A Dockerfile and compose to provide and initialize basic test local infrastructure (rabbittmq, timescaledb)
  and lauch the app without any install (other than docker)
- A ingest/ingest-rust project with three binaries
  - a small python utility to generate dummy json logs
  - a light agent that transfers stdin into rabbitmq (logdog-producer)
  - an ingest json logs utility which reads rabbitmq and inserts to timescaledb at amazing speeds (logdog-consumer, measured 30k logs per second) 
- A web engine to query the logs, that have a backend (logsearcher-server) and a frontend in vue (logsearcher)

Used together, they can make a small DIY solution to gather logs from several sources and providing analytics display, 
with enough performance to handle over a million log a day on a reasonably small virtual host.

## Usage

- Run the log infrastructure through `docker compose -d`
- Run logsearcher server through `cargo run` in  logsearcher-server directory
- Run logsearcher front through `npm run dev` in logsearcher directory
- Start ingesting some logs, by running teh consumer in ingest/rust (`cargo run --bin logdog-consumer` then, in src, `python generate_logs.py | cargo run --bin logdog-producer`)
- Explore them in the view.

## Contributing

Request features or fixes through this github issues.