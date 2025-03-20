# rquest-rb

A high-performance HTTP client for Ruby with TLS fingerprinting capabilities. This gem is a Ruby binding to the blazing-fast Rust [`rquest`](https://github.com/0x676e67/rquest) HTTP client.

## Features

- Fast performance using Rust's `rquest` HTTP client
- API compatible with [http.rb](https://github.com/httprb/http)
- Browser TLS fingerprinting support
- HTTP/2 support
- Thread-safe

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'rquest-rb'
```

And then execute:

```
$ bundle
```

Or install it yourself as:

```
$ gem install rquest-rb
```

## Usage

This gem is designed as a drop-in replacement for the http.rb gem. Here are some examples:

### Basic GET Request

```ruby
require 'rquest-rb'

# Simple GET request
response = HTTP.get("https://httpbin.org/get")

puts response.status  # => 200
puts response.body    # => JSON response body
```

### Working with Headers

```ruby
# Adding custom headers
response = HTTP
  .headers(accept: "application/json", user_agent: "My App/1.0")
  .get("https://httpbin.org/headers")

# Chain methods together
response = HTTP
  .headers(accept: "application/json")
  .follow(true)  # enable redirects
  .get("https://httpbin.org/get")
```

### Using a Proxy

```ruby
# Using a proxy for requests
response = HTTP
  .proxy("http://proxy.example.com:8080")
  .get("https://httpbin.org/get")

# Chain proxy with other options
response = HTTP
  .proxy("http://proxy.example.com:8080")
  .headers(accept: "application/json")
  .get("https://httpbin.org/get")

# Using proxy with authentication
response = HTTP
  .proxy("http://username:password@proxy.example.com:8080")
  .get("https://httpbin.org/get")
```

### Making POST Requests

```ruby
# POST with a body
response = HTTP.post(
  "https://httpbin.org/post",
  body: "This is the request body"
)

# POST with JSON
response = HTTP
  .headers(content_type: "application/json")
  .post(
    "https://httpbin.org/post",
    body: JSON.generate({ name: "Example", value: 123 })
  )
```

### Other HTTP Methods

```ruby
# PUT request
HTTP.put("https://httpbin.org/put", body: "Updated content")

# DELETE request
HTTP.delete("https://httpbin.org/delete")

# HEAD request
HTTP.head("https://httpbin.org/get")

# PATCH request
HTTP.patch("https://httpbin.org/patch", body: "Patched content")
```

## Benchmarks

rquest-rb is designed to be a high-performance alternative to other Ruby HTTP clients. Here's how it compares:

### Running Benchmarks

The project includes benchmarks to compare rquest-rb with other popular Ruby HTTP clients.

```
$ bundle exec ruby benchmark/http_clients_benchmark.rb
```

This will run a benchmark making 5,000 requests to a test endpoint with concurrency, comparing multiple HTTP clients.

Recent benchmark results (sequential requests to https://serpapi.com/robots.txt):
```
Ruby 2.7.8p225 (2023-03-30 revision 1f4d455848) [arm64-darwin23]

Comparison:
            typhoeus:       59.2 i/s
           rquest-rb:       23.4 i/s - 2.53x  slower
             http.rb:       12.0 i/s - 4.92x  slower
                curb:       11.8 i/s - 5.02x  slower
               httpx:        9.3 i/s - 6.35x  slower
```

As shown above, Typhoeus is the fastest client, processing requests approximately 2.5x faster than rquest-rb. Rquest-rb is still significantly faster than http.rb (4.92x), curb (5.02x), and httpx (6.35x) in sequential operations.

### Benchmark History

Benchmarks are automatically run on every push to the master branch using GitHub Actions. This allows us to track performance over time and ensure rquest-rb maintains its performance advantage.

#### Performance Trend Visualization

Benchmark charts are generated for multiple Ruby versions (2.7, 3.0, 3.1, 3.2, 3.3) to track performance across different Ruby implementations.

##### Combined Performance Comparison
The following chart shows how rquest-rb compares to other HTTP clients across all tested Ruby versions:

![Combined HTTP Client Performance](https://github.com/0x676e67/rquest-rb/raw/main/docs/assets/combined_time_chart.png)

As shown in our latest benchmarks, Typhoeus is the fastest client, with rquest-rb being the second fastest. Both significantly outperform HTTP.rb, Curb, and HTTPX across all Ruby versions.

##### Ruby 2.7 (default)
![Request Time Benchmark Chart (Ruby 2.7)](https://github.com/0x676e67/rquest-rb/raw/main/docs/assets/time_chart-2.7.png)
![Requests Per Second Benchmark Chart (Ruby 2.7)](https://github.com/0x676e67/rquest-rb/raw/main/docs/assets/rps_chart-2.7.png)

For performance charts of other Ruby versions, see the [benchmark summary page](https://github.com/0x676e67/rquest-rb/blob/main/docs/assets/benchmark_summary.md).

*Note: These charts are automatically generated during CI runs. The latest charts can be found in the GitHub Actions artifacts.*

#### Raw Benchmark Data

You can find historical benchmark results in the GitHub Actions artifacts. Each run stores:
- A detailed benchmark result for each Ruby version
- CSV files with historical benchmark data for each Ruby version
- Graphviz charts in PNG and SVG formats

To visualize benchmark history, download the `benchmark-history-{ruby_version}.csv` artifact and use the provided script:

```
$ script/visualize_benchmarks.rb -f benchmark-history-2.7.csv
```

Options:
- `-f, --file FILE` - Specify the CSV file path
- `-m, --metric TYPE` - Metric to visualize (time or requests_per_second)
- `-l, --limit NUM` - Limit to last N entries

## Development

After checking out the repo, install dependencies and build the extension:

```
$ bundle install
$ bundle exec rake compile
```

To run tests:

```
$ bundle exec rake test
```

## Contributing

1. Fork it
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -am 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Create a new Pull Request

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT). 