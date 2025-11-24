CREATE DATABASE engine;

\connect engine;


CREATE TABLE IF NOT EXISTS orders
(
    order_id      INTEGER     NOT NULL PRIMARY KEY,
    client_id     INTEGER     NOT NULL,
    instrument    VARCHAR(16) NOT NULL,
    side          SMALLINT    NOT NULL,
    px            INTEGER     NOT NULL,
    qty           INTEGER     NOT NULL,
    qty_rem       INTEGER     NOT NULL,
    time_in_force SMALLINT    NOT NULL,
    ack_time      BIGINT      NOT NULL
);


CREATE TABLE IF NOT EXISTS trades
(
    trade_id      INTEGER  NOT NULL PRIMARY KEY,
    trade_seq     INTEGER  NOT NULL,
    bid_client_id INTEGER  NOT NULL,
    bid_order_id  INTEGER  NOT NULL,
    bid_order_px  INTEGER  NOT NULL,
    bid_fill_type SMALLINT NOT NULL,
    ask_client_id INTEGER  NOT NULL,
    ask_order_id  INTEGER  NOT NULL,
    ask_order_px  INTEGER  NOT NULL,
    ask_fill_type SMALLINT NOT NULL,
    instrument    CHAR(16) NOT NULL,
    exec_px       INTEGER  NOT NULL,
    exec_qty      INTEGER  NOT NULL,
    exec_type     SMALLINT NOT NULL,
    exec_ns       BIGINT   NOT NULL
);