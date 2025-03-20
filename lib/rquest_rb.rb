require_relative "rquest_rb/version"

module Rquest
  module HTTP
    # Methods are defined by the native extension
  end
end

# Create a top-level HTTP constant for convenience
HTTP = Rquest::HTTP

# Tries to require the extension for the given Ruby version first
begin
  RUBY_VERSION =~ /(\d+\.\d+)/
  require "rquest/#{Regexp.last_match(1)}/rquest_rb"
rescue LoadError
  require "rquest/rquest_rb"
end 