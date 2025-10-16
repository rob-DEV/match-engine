docker run --name pg -e POSTGRES_PASSWORD=password -p 5432:5432 -d postgres
docker exec -it pg psql -U postgres -c "CREATE DATABASE engine;"

docker exec -it pg psql -U postgres -d mydatabase -c "
CREATE TABLE IF NOT EXISTS SYMBOL (
      id SERIAL PRIMARY KEY,
      symbol VARCHAR(8) NOT NULL
);"

docker exec -it pg psql -U postgres -d mydatabase -c "
CREATE TABLE IF NOT EXISTS ORDER (
      id SERIAL PRIMARY KEY,            -- unique trade ID
      symbol TEXT NOT NULL,             -- trading symbol, e.g., "AAPL"
      side TEXT NOT NULL,               -- "BUY" or "SELL"
      quantity NUMERIC(20, 4) NOT NULL, -- number of shares/contracts
      price NUMERIC(20, 4) NOT NULL,    -- execution price
      trade_time TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- timestamp of trade
      trader_id TEXT,                   -- ID of trader who executed the trade
      venue TEXT,                        -- exchange or venue
      order_type TEXT,                   -- e.g., "LIMIT", "MARKET"
      status TEXT DEFAULT 'EXECUTED'     -- status of the trade
);"

docker exec -it pg psql -U postgres -d mydatabase -c "
CREATE TABLE IF NOT EXISTS users (
      id SERIAL PRIMARY KEY,            -- unique trade ID
      symbol TEXT NOT NULL,             -- trading symbol, e.g., "AAPL"
      side TEXT NOT NULL,               -- "BUY" or "SELL"
      quantity NUMERIC(20, 4) NOT NULL, -- number of shares/contracts
      price NUMERIC(20, 4) NOT NULL,    -- execution price
      trade_time TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- timestamp of trade
      trader_id TEXT,                   -- ID of trader who executed the trade
      venue TEXT,                        -- exchange or venue
      order_type TEXT,                   -- e.g., "LIMIT", "MARKET"
      status TEXT DEFAULT 'EXECUTED'     -- status of the trade
);"


##[derive(Encode, Decode, PartialEq, Debug)]
#pub struct TradeExecution {
#    pub trade_id: u32,
#    pub trade_seq: u32,
#    pub bid_client_id: u32,
#    pub ask_client_id: u32,
#    pub bid_order_id: u32,
#    pub ask_order_id: u32,
#    pub fill_qty: u32,
#    pub px: u32,
#    pub execution_time: u64,
#}