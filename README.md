# Events service

This project consists of two portions:
- the url shortener, written in kotlin with spring
- ** this project, the events server, a dead simple server that record events, written in rust using actix.**

The main purpose of this project is to demonstrate events handling for short url.
It records the number of times a url is clicked and how many times a url was created.

The id we use for this service is a snowflake.

## How to run the program

You need the latest version of rust(1.8.0). Please install this before
attempting to run this program!

Start the database.
```shell
./scripts/init_db.sh
```

Once cargo is installed, run the following

```shell
cargo build
cargo test --color=always --package events --test main
cargo run
```
