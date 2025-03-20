require 'test/unit'
require 'rquest_rb'
require 'json'

class RquestTest < Test::Unit::TestCase
  def setup
    @client = HTTP::Client.new
  end

  def test_basic_get_request
    response = HTTP.get("https://httpbin.org/get")
    assert_equal(200, response.status)
    assert_kind_of(String, response.body)
    assert_kind_of(Hash, response.headers)
  end

  def test_client_instance_get_request
    response = @client.get("https://httpbin.org/get")
    assert_equal(200, response.status)
    assert_kind_of(String, response.body)
  end

  def test_random_user_agent
    # Make multiple requests and verify different user agents are used
    agents = []
    5.times do
      client = HTTP::Client.new
      response = client.get("https://httpbin.org/headers")
      assert_equal(200, response.status)
      
      body = JSON.parse(response.body)
      user_agent = body["headers"]["User-Agent"]
      agents << user_agent
    end
    
    # Check that we got at least 2 different user agents (should be random)
    assert agents.uniq.size > 1, "Expected different random user agents, but got: #{agents.uniq}"
  end

  def test_headers
    response = HTTP
      .headers(accept: "application/json", user_agent: "Test Client")
      .get("https://httpbin.org/headers")
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal("application/json", body["headers"]["Accept"])
    # Don't test for specific user agent as we're using real browser impersonation
    # and the value will be different each time
  end

  def test_post_request
    response = HTTP.post(
      "https://httpbin.org/post",
      body: "test body"
    )
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal("test body", body["data"])
  end

  def test_post_json
    response = HTTP
      .headers(content_type: "application/json")
      .post(
        "https://httpbin.org/post",
        body: JSON.generate({ name: "test", value: 123 })
      )
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal({ "name" => "test", "value" => 123 }, JSON.parse(body["data"]))
  end

  def test_put_request
    response = HTTP.put(
      "https://httpbin.org/put",
      body: "updated content"
    )
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal("updated content", body["data"])
  end

  def test_delete_request
    response = HTTP.delete("https://httpbin.org/delete")
    assert_equal(200, response.status)
  end

  def test_head_request
    response = HTTP.head("https://httpbin.org/get")
    assert_equal(200, response.status)
    assert_empty(response.body)
  end

  def test_patch_request
    response = HTTP.patch(
      "https://httpbin.org/patch",
      body: "patched content"
    )
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal("patched content", body["data"])
  end

  def test_follow_redirects
    response = HTTP
      .follow(true)
      .get("https://httpbin.org/redirect/1")
    
    assert_equal(200, response.status)
    assert_equal("https://httpbin.org/get", response.uri)
  end

  def test_no_follow_redirects
    response = HTTP
      .follow(false)
      .get("https://httpbin.org/redirect/1")
    
    assert_equal(302, response.status)
    assert_equal("https://httpbin.org/redirect/1", response.uri)
  end

  def test_response_methods
    response = HTTP.get("https://httpbin.org/get")
    
    assert_kind_of(Integer, response.status)
    assert_kind_of(String, response.body)
    assert_kind_of(Hash, response.headers)
    assert_kind_of(String, response.uri)
    assert_kind_of(String, response.to_s)
  end

  def test_content_type
    response = HTTP
      .headers(accept: "application/json")
      .get("https://httpbin.org/get")
    
    assert_equal("application/json", response.content_type)
  end
  
  def test_bing_search_results
    # Create a client with a common browser user agent to avoid being blocked
    client = HTTP::Client.new
    
    # Fetch Bing search results for "Coffee"
    response = client
      .headers(accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
      .get("https://www.bing.com/search?form=QBRE&q=Coffee&lq=0&rdr=1")
    
    assert_equal(200, response.status)
    
    # Convert response body to lowercase for case-insensitive checks
    body_text = response.body.downcase
    
    # Check for different types of results
    has_organic_results = body_text.include?("web results") || body_text.include?("all results")
    has_shopping = body_text.include?("shopping") || body_text.include?("product")
    has_knowledge_graph = body_text.include?("wikipedia") || body_text.include?("fact")
    
    # Assert that we have more than just organic results
    assert has_organic_results, "No organic results found"
    
    # We expect either shopping results or knowledge graph to be present
    assert has_shopping || has_knowledge_graph, 
      "Expected rich results beyond organic listings. " +
      "Shopping: #{has_shopping}, Knowledge Graph: #{has_knowledge_graph}"
  end
  
  def test_tls_fingerprinting
    # Create multiple clients to test fingerprint randomization
    fingerprints = []
    
    3.times do
      client = HTTP::Client.new
      
      # Make request to TLS fingerprinting service
      response = client.get("https://tls.peet.ws/api/all")
      assert_equal(200, response.status)
      
      # Parse the JSON response
      data = JSON.parse(response.body)
      
      # Verify TLS data structure
      assert_kind_of(Hash, data["tls"])
      assert_kind_of(Array, data["tls"]["ciphers"])
      assert(data["tls"]["ciphers"].size > 0, "Expected TLS ciphers to be present")
      
      # Check for JA3 and JA4 fingerprints
      assert_not_nil(data["tls"]["ja3"], "JA3 fingerprint should be present")
      assert_not_nil(data["tls"]["ja3_hash"], "JA3 hash should be present")
      assert_not_nil(data["tls"]["ja4"], "JA4 fingerprint should be present")
      
      # Store fingerprints for comparison
      fingerprints << {
        ja3: data["tls"]["ja3_hash"],
        ja4: data["tls"]["ja4"]
      }
      
      # Verify TLS version is modern
      tls_version = data["tls"]["tls_version_negotiated"]
      assert(["771", "772"].include?(tls_version), 
        "Expected modern TLS version (TLS 1.2 or 1.3), got: #{tls_version}")
    end
    
    # Check for fingerprint randomization
    # Either JA3 or JA4 should have some variation across requests 
    ja3_fingerprints = fingerprints.map { |f| f[:ja3] }.uniq
    ja4_fingerprints = fingerprints.map { |f| f[:ja4] }.uniq
    
    assert(ja3_fingerprints.size > 1 || ja4_fingerprints.size > 1,
      "Expected fingerprint randomization, but got identical fingerprints across requests")
  end
end 