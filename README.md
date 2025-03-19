# rquest-rb

A high-performance HTTP client for Ruby with TLS fingerprinting capabilities. This gem is a Ruby binding to the blazing-fast Rust [rquest](https://github.com/0x676e67/rquest) HTTP client.

## Features

- Fast performance using Rust's rquest HTTP client
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