# Trade Matching Engine

Experimental trade matching engine written in Rust.

## Features

- Tokio RT Match Engine Gateway
- Order Entry / Market Data API
- API -> Engine Messaging System

## Usage

### Startup

Start the Matching Engine & API

```
 cargo run --release --bin engine
 cargo run --release --bin api
```

1. The engine runs on port `3000` by default.
1. The engine api runs on port `3001` by default.
1. The engines order book is empty on start up.

### Order Entry

Orders can be submitted via a `POST` request to the API.

```
curl --location 'http://localhost:3000/order' \
--header 'Content-Type: application/json' \
--data '{
    "NewOrder": {
        "action": "SELL",
        "px": 50,
        "qty": 1000
    }
}'
```

The engine will ACK new Orders.

```
{
    "NewOrderAck": {
        "ack_time": 1706441585242165679,
        "action": "SELL",
        "id": 451748013,
        "px": 100,
        "qty": 1003
    }
}
```

### Requesting Market Data

Market data can be requested from the engine in two forms `FullSnapshot` and `TopOfBook`

```
curl --location 'http://localhost:3000/md' \
--header 'Content-Type: application/json' \
--data '{
    "MarketDataRequest": {
        "snapshot_type": "FullSnapshot"
    }
}'
```

The engine will provide the latest market data, currently there is no state persistence.

```
{
    "MarketDataResponse": {
        "FullSnapshot": {
            "asks": [
                {
                    "px": 529,
                    "qty": 1058
                },
                {
                    "px": 527,
                    "qty": 1054
                },
                {
                    "px": 526,
                    "qty": 2104
                },
                {
                    "px": 524,
                    "qty": 1048
                }
            ],
            "bids": [
                {
                    "px": 509,
                    "qty": 509
                },
                {
                    "px": 495,
                    "qty": 495
                },
                {
                    "px": 466,
                    "qty": 3728
                }
            ],
            "snapshot_type": "FullSnapshot"
        }
    }
}
```

## Building

If desired, you can build Rust-template yourself. You will need a working `Rust` and `Cargo`
setup. [Rustup](https://rustup.rs/) is the simplest way to set this up on either Windows, Mac or Linux.

Once the prerequisites have been installed, compilation on your native platform is as simple as running the following in
a terminal:

```
cargo build --release
```

## License

Rust-template itself is licensed under the [MIT license](LICENSE) and includes this as the default project license.