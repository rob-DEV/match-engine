wrk.method = "POST"
wrk.body   = "{\"NewOrder\":{\"action\":\"SELL\",\"px\":100,\"qty\":12233}}"
wrk.headers["Content-Type"] = "application/json"
