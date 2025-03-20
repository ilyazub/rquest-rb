require "bundler/gem_tasks"
require "rake/testtask"
require "rake/extensiontask"

GEMSPEC = Gem::Specification.load("rquest-rb.gemspec") || abort("Could not load rquest-rb.gemspec")

# Define supported platforms
SUPPORTED_PLATFORMS = [
  "x86_64-linux",
  "x86_64-darwin",
  "arm64-darwin",
  "x64-mingw32",
  "x64-mingw-ucrt",
]

# Helper to check if Docker/Podman is available
def container_runtime_available?
  system("which docker > /dev/null 2>&1") || system("which podman > /dev/null 2>&1")
end

# Helper to build for a specific platform
def build_for_platform(platform)
  puts "Building for platform: #{platform}"
  sh 'bundle', 'exec', 'rb-sys-dock', '--platform', platform, '--build'
end

# Define the extension task
Rake::ExtensionTask.new("rquest_rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/rquest"
  ext.ext_dir = "ext/rquest_rb"
  ext.cross_compile = true
  ext.cross_platform = SUPPORTED_PLATFORMS
end

# Build the native extension for the current platform
desc "Build the native extension for the current platform"
task :compile do
  sh "bundle"
  sh "bundle exec rake build"
end

# Build the gem for the current platform
desc "Build the gem for the current platform"
task :gem => :compile

# Cross-compile and build native gems for all supported platforms
namespace "gem" do
  desc "Build native gems for all supported platforms"
  task "all-platforms" => [:clean] do
    require "rake_compiler_dock"
    
    unless container_runtime_available?
      puts "⚠️  Warning: Docker or Podman is required for cross-compilation but not found."
      puts "Please install Docker or Podman to build for all platforms."
      puts "Continuing with only the current platform..."
      Rake::Task[:compile].invoke
      next
    end

    # Build native gems for all platforms
    SUPPORTED_PLATFORMS.each do |platform|
      build_for_platform(platform)
    end
  end

  desc "Build native extension for a specific platform (e.g., `rake 'gem:native[x86_64-linux]'`)"
  task :native, [:platform] do |_t, args|
    platform = args[:platform]
    if platform.nil? || platform.empty?
      abort "Platform must be specified, e.g., rake 'gem:native[x86_64-linux]'"
    end
    
    unless container_runtime_available?
      abort "Docker or Podman is required for cross-compilation but not found. Please install one of them."
    end
    
    build_for_platform(platform)
  end
end

require "ruby_memcheck"

namespace :test do
  desc "Run memory checks using ruby_memcheck"
  task :memcheck do
    RubyMemcheck.config do |config|
      # Configure suppressions for known false positives
      config.suppressions = [
        # Add specific suppressions if needed
      ]
      
      # Set the test command to run
      config.binary = "ruby"
      config.command = "test/memory_leak_test.rb"
      
      # Configure Valgrind options
      config.valgrind.options = %w[
        --leak-check=full
        --show-leak-kinds=all
        --track-origins=yes
        --error-exitcode=1
      ]
    end

    RubyMemcheck.run
  end

  desc "Run quick memory check"
  task :memcheck_quick do
    RubyMemcheck.config do |config|
      config.binary = "ruby"
      config.command = "test/memory_leak_test.rb --quick"
      config.valgrind.options = %w[--leak-check=full]
    end

    RubyMemcheck.run
  end
end

# Default memory check task
task memcheck: "test:memcheck" 

# Development tasks
task :fmt do
  sh 'cargo', 'fmt'
end

task :rust_test do
  sh "cargo test"
end

# Run Ruby tests
Rake::TestTask.new(:ruby_test) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.libs << File.expand_path("lib", __dir__)  # Add the lib directory to load path
  t.libs << File.expand_path("lib/rquest", __dir__)  # Add the native extension directory
  t.test_files = FileList["test/**/*_test.rb"]
  t.deps << :compile  # Make sure the native extension is built before running tests
end

task test: %i[rust_test memcheck ruby_test]

namespace :benchmark do
  desc "Run HTTP clients benchmark"
  task :http_clients do
    puts "Running HTTP clients benchmark..."
    ruby 'benchmark/http_clients_benchmark.rb'
  end
end

desc "Run all benchmarks"
task :benchmark => ['benchmark:http_clients']

task default: %i[compile test benchmark]
