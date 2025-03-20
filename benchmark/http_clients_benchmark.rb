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

# Sequential benchmarks
puts "\nSequential requests (one at a time):"
Benchmark.ips do |x|
  x.config(time: 10, warmup: 10)

  x.report("curb") do
    curl = Curl::Easy.new(URL)
    curl.perform
    curl.body_str
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

# Batch request method for rquest-rb
def rquest_batch_requests(url, count, concurrency)
  results = []
  threads = []
  
  (0...count).each_slice(count / concurrency) do |batch|
    threads << Thread.new do
      batch.each do |_|
        results << Rquest::HTTP.get(url).to_s
      end
    end
  end
  
  threads.each(&:join)
  results
end

# Batch request method for http.rb
def http_batch_requests(url, count, concurrency)
  results = []
  threads = []
  
  (0...count).each_slice(count / concurrency) do |batch|
    threads << Thread.new do
      batch.each do |_|
        results << HTTP.get(url).to_s
      end
    end
  end
  
  threads.each(&:join)
  results
end

# Batch request method for curb
def curb_batch_requests(url, count, concurrency)
  results = []
  threads = []
  
  (0...count).each_slice(count / concurrency) do |batch|
    threads << Thread.new do
      batch.each do |_|
        results << Curl.get(url)body
      end
    end
  end
  
  threads.each(&:join)
  results
end

# Batch request method for typhoeus
def typhoeus_batch_requests(url, count, concurrency)
  results = []
  threads = []
  
  (0...count).each_slice(count / concurrency) do |batch|
    threads << Thread.new do
      batch.each do |_|
        results << Typhoeus.get(url).body
      end
    end
  end
  
  threads.each(&:join)
  results
end

# Batch request method for httpx
def httpx_batch_requests(url, count, concurrency)
  results = []
  threads = []
  
  (0...count).each_slice(count / concurrency) do |batch|
    threads << Thread.new do
      batch.each do |_|
        results << HTTPX.get(url).body.to_s
      end
    end
  end
  
  threads.each(&:join)
  results
end