function randomised_side ()
    if math.random(1, 10) % 2 == 0 then
        return "BUY"
    else
        return "SELL"
    end
end

request = function()
    wrk.method = "POST"
    wrk.body = string.format("{\"NewOrder\":{\"action\":\"%s\",\"px\":%d,\"qty\":%d}}", randomised_side(), math.random(1, 100), math.random(1, 100))
    wrk.headers["Content-Type"] = "application/json"
    return wrk.format("POST")
end