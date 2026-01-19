class Mote < Formula
  desc "A fine-grained snapshot management tool for projects"
  homepage "https://github.com/shabaraba/mote"
  version "0.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/shabaraba/mote/releases/download/v#{version}/mote-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_ARM64"
    else
      url "https://github.com/shabaraba/mote/releases/download/v#{version}/mote-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/shabaraba/mote/releases/download/v#{version}/mote-#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_LINUX_ARM64"
    else
      url "https://github.com/shabaraba/mote/releases/download/v#{version}/mote-#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "mote"
  end

  test do
    system "#{bin}/mote", "--version"
  end
end
