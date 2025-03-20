require_relative "lib/rquest_rb/version"

Gem::Specification.new do |spec|
  spec.name          = "rquest-rb"
  spec.version       = Rquest::HTTP::VERSION
  spec.authors       = ["SerpApi Team"]
  spec.email         = ["contact@serpapi.com"]

  spec.summary       = "Ruby bindings for the rquest Rust HTTP client"
  spec.description   = "A high-performance drop-in replacement for http.rb gem (HTTP client for Ruby) with TLS fingerprinting capabilities"
  spec.homepage      = "https://github.com/ilyazub/rquest-rb"
  spec.license       = "MIT"
  spec.required_ruby_version = ">= 2.6.0"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = spec.homepage
  spec.metadata["changelog_uri"] = "#{spec.homepage}/blob/main/CHANGELOG.md"

  # Include all necessary files
  spec.files = Dir.glob("{lib,ext}/**/*.{rb,rs,toml}") + %w[README.md LICENSE.txt]
  spec.require_paths = ["lib"]
  
  # Configure the extension
  spec.extensions = ["ext/rquest_rb/extconf.rb"]
  spec.extra_rdoc_files = ["README.md", "LICENSE.txt"]

  # needed until rubygems supports Rust support is out of beta
  spec.add_dependency "rb_sys", "~> 0.9.39"

  # only needed when developing or packaging your gem
  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2.0"
  spec.add_development_dependency "minitest", "~> 5.0"
end 