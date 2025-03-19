require_relative "rquest_rb/version"

module Rquest
  module HTTP
    class << self
      def headers(headers = {})
        Client.new.with_headers(headers)
      end
  
      def follow(follow = true)
        Client.new.follow(follow)
      end
  
      def get(url)
        Client.new.get(url)
      end
  
      def post(url, body = nil)
        Client.new.post(url, body)
      end
  
      def put(url, body = nil)
        Client.new.put(url, body)
      end
  
      def delete(url)
        Client.new.delete(url)
      end
  
      def head(url)
        Client.new.head(url)
      end
  
      def patch(url, body = nil)
        Client.new.patch(url, body)
      end
    end
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