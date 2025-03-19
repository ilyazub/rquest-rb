require 'test/unit'
require 'rquest-rb'
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

  def test_headers
    response = HTTP
      .headers(accept: "application/json", user_agent: "Test Client")
      .get("https://httpbin.org/headers")
    
    assert_equal(200, response.status)
    body = JSON.parse(response.body)
    assert_equal("application/json", body["headers"]["Accept"])
    assert_equal("Test Client", body["headers"]["User-Agent"])
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
end 