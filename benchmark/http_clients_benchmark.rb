#!/usr/bin/env ruby
# frozen_string_literal: true

require "benchmark/ips"
require "curb"

# First, load the original http.rb
require "http"

# Now load rquest_rb which no longer overrides HTTP
require "rquest_rb"

URL = "https://serpapi.com/robots.txt"
REQUESTS = 1_000
CONCURRENCY = 10 # Number of concurrent requests

puts "Benchmarking HTTP clients making #{REQUESTS} requests to #{URL}"
puts "--------------------------------------------------------"

# Sequential benchmarks
puts "\nSequential requests (one at a time):"
Benchmark.ips do |x|
  x.config(time: 10, warmup: 2)

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
        curl = Curl::Easy.new(url)
        curl.perform
        results << curl.body_str
      end
    end
  end
  
  threads.each(&:join)
  results
end

# Full benchmark timing each client making 1000 requests with concurrency
puts "\nMaking #{REQUESTS} requests with concurrency of #{CONCURRENCY}:"
puts "--------------------------------------------------------"

puts "\nCurb:"
start_time = Time.now
curb_batch_requests(URL, REQUESTS, CONCURRENCY)
curb_time = Time.now - start_time
puts "Time: #{curb_time.round(2)} seconds (#{(REQUESTS / curb_time).round(2)} req/s)"

puts "\nHTTP.rb:"
start_time = Time.now
http_batch_requests(URL, REQUESTS, CONCURRENCY)
http_time = Time.now - start_time
puts "Time: #{http_time.round(2)} seconds (#{(REQUESTS / http_time).round(2)} req/s)"

puts "\nRquest-rb:"
start_time = Time.now
rquest_batch_requests(URL, REQUESTS, CONCURRENCY)
rquest_time = Time.now - start_time
puts "Time: #{rquest_time.round(2)} seconds (#{(REQUESTS / rquest_time).round(2)} req/s)"

puts "\nComparison:"
puts "--------------------------------------------------------"
puts "Curb:      #{curb_time.round(2)} seconds (#{(REQUESTS / curb_time).round(2)} req/s)"
puts "HTTP.rb:   #{http_time.round(2)} seconds (#{(REQUESTS / http_time).round(2)} req/s)"
puts "Rquest-rb: #{rquest_time.round(2)} seconds (#{(REQUESTS / rquest_time).round(2)} req/s)"

fastest = [["Curb", curb_time], ["HTTP.rb", http_time], ["Rquest-rb", rquest_time]].min_by { |_, time| time }
puts "\nFastest client: #{fastest[0]} (#{(fastest[1] * 1000 / REQUESTS).round(2)} ms per request)" 