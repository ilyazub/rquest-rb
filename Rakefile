require "bundler/gem_tasks"
require "rake/testtask"
require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("rquest-rb.gemspec") || abort("Could not load rquest-rb.gemspec")

RbSys::ExtensionTask.new("rquest-rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/rquest"
  ext.ext_dir = "ext/rquest_rb"
end

# Build native gems for the current platform
task "gem:native" do
  require "rake_compiler_dock"
  sh "bundle"
  sh "bundle exec rake gem"
end

# Cross-compile and build native gems for all supported platforms
namespace "gem" do
  task "all-platforms" => [:clean] do
    require "rake_compiler_dock"
    platform_patterns = [
      "x86_64-linux",
      "x86_64-darwin",
      "arm64-darwin",
      "x64-mingw32",
      "x64-mingw-ucrt",
    ]

    # Check if Docker or Podman is available
    docker_available = system("which docker > /dev/null 2>&1") || system("which podman > /dev/null 2>&1")
    
    unless docker_available
      puts "⚠️  Warning: Docker or Podman is required for cross-compilation but not found."
      puts "Please install Docker or Podman to build for all platforms."
      puts "Continuing with only the current platform..."
      sh "bundle exec rake build"
      next
    end

    # Build native gems for all platforms
    platform_patterns.each do |platform|
      puts "Building for platform: #{platform}"
      sh 'bundle', 'exec', 'rb-sys-dock', '--platform', platform, '--build'
    end
  end

  desc "Build native extension for a given platform (i.e. `rake 'gem:native[x86_64-linux]'`)"
  task :native, [:platform] do |_t, args|
    platform = args[:platform]
    if platform.nil? || platform.empty?
      abort "Platform must be specified, e.g., rake 'gem:native[x86_64-linux]'"
    end
    
    # Check if Docker or Podman is available
    docker_available = system("which docker > /dev/null 2>&1") || system("which podman > /dev/null 2>&1")
    
    unless docker_available
      abort "Docker or Podman is required for cross-compilation but not found. Please install one of them."
    end
    
    sh 'bundle', 'exec', 'rb-sys-dock', '--platform', platform, '--build'
  end
end

task :fmt do
  sh 'cargo', 'fmt'
end

task :cargo_test do
  sh "cargo test"
end

Rake::TestTask.new(:ruby_test) do |t|
  t.libs << "test"
  t.libs << "lib"
  t.test_files = FileList["test/**/*_test.rb"]
end

task test: %i[ruby_test cargo_test]

task build: :compile

task default: %i[compile test]