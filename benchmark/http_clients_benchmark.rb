#!/usr/bin/env ruby

require "benchmark/ips"
require "curb"
require "typhoeus"
require "httpx"

# First, load the original http.rb
require "http"

# Now load rquest_rb which no longer overrides HTTP
require "rquest_rb"

URL = "https://serpapi.com/robots.txt"
REQUESTS = 5_000
CONCURRENCY = 10 # Number of concurrent requests

puts "Benchmarking HTTP clients making #{REQUESTS} requests to #{URL}"
puts "--------------------------------------------------------"

puts "\nSequential requests (one at a time):"
Benchmark.ips do |x|
  x.config(time: 10, warmup: 10)

  x.report("curb") do
    Curl.get(URL).body
  end

  x.report("http.rb") do
    HTTP.get(URL).to_s
  end

  x.report("rquest-rb") do
    Rquest::HTTP.get(URL).to_s
  end
  
  x.report("typhoeus") do
    Typhoeus.get(URL).body
  end
  
  x.report("httpx") do
    HTTPX.get(URL).body.to_s
  end

  x.compare!
end