#!/bin/zsh

seq 1000000 | parallel --max-args 0 --jobs 500 "curl --location 'http://localhost:3000/order' \
                                          --header 'Content-Type: application/json' \
                                          --data '{
                                              \"token\": \"938c2cc0dcc05f2b68c4287040cfcf71\",
                                              \"symbol\": \"ABC\",
                                              \"qty\": \"10.0\",
                                              \"px\": \"15.0\"
                                          }'"