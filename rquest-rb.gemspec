# frozen_string_literal: true

Gem::Specification.new do |spec|
  spec.name          = "rquest-rb"
  spec.version       = "0.1.0"
  spec.authors       = ["Your Name"]
  spec.email         = ["your.email@example.com"]

  spec.summary       = "Ruby bindings for the rquest Rust HTTP client"
  spec.description   = "A high-performance HTTP client for Ruby with TLS fingerprinting capabilities"
  spec.homepage      = "https://github.com/yourusername/rquest-rb"
  spec.license       = "MIT"
  spec.required_ruby_version = ">= 2.6.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/blob/main/CHANGELOG.md"

  spec.files = Dir.glob("{lib,ext}/**/*.{rb,rs}") + %w[README.md LICENSE.txt]
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/rquest_rb/extconf.rb"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys", "~> 0.9.39"

  # only needed when developing or packaging your gem
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2.0"
  spec.add_development_dependency "test-unit", "~> 3.5"
end 