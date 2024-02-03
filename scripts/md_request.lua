wrk.method = "POST"
wrk.body = "{\"MarketDataRequest\": {\"snapshot_type\": \"FullSnapshot\"}}"
wrk.headers["Content-Type"] = "application/json"