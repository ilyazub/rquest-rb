require "bundler/gem_tasks"
require "rake/testtask"
require "rb_sys/extensiontask"

GEMSPEC = Gem::Specification.load("rquest-rb.gemspec") || abort("Could not load rquest-rb.gemspec")

RbSys::ExtensionTask.new("rquest-rb", GEMSPEC) do |ext|
  ext.lib_dir = "lib/rquest"
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

    desc "Build native extension for a given platform (i.e. `rake 'native[x86_64-linux]'`)"
    task :native, [:platform] do |_t, platform:|
      sh 'bundle', 'exec', 'rb-sys-dock', '--platform', platform, '--build'
    end
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