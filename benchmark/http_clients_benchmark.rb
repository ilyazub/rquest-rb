#!/usr/bin/env ruby

require "benchmark/ips"
require "curb"
require "typhoeus"
require "httpx"
require "http"
require "rquest_rb"

URL = "https://serpapi.com/robots.txt"

Benchmark.ips do |x|
  x.config(warmup: 5, time: 60)

  x.report("curb") do
    Curl.get(URL).status
  end

  x.report("http.rb") do
    HTTP.get(URL).status
  end

  x.report("rquest-rb") do
    Rquest::HTTP.get(URL).code
  end
  
  x.report("typhoeus") do
    Typhoeus.get(URL).code
  end
  
  x.report("httpx") do
    HTTPX.get(URL).status
  end

  x.compare!
end