# Homebrew formula for INVAR. Lives here as the source of truth; the
# founder copies it into Ramesh-35/homebrew-tap as Formula/invar.rb.
# Install: brew install Ramesh-35/tap/invar
# Hashes below were re-downloaded and measured on 2026-10-12, not
# copied from the release page. Regenerate for each release: download
# the three tarballs, sha256sum them, update url/sha256/version.
class Invar < Formula
  desc "Checks cross-repository contracts; a skip is never a pass"
  homepage "https://github.com/Ramesh-35/invar"
  version "0.1.0-alpha.21"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.21/invar-v0.1.0-alpha.21-aarch64-apple-darwin.tar.gz"
      sha256 "a9e72f27b8d36848d7027a122190174174a5a386736d6b090e72ea113df20b2f"
    else
      url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.21/invar-v0.1.0-alpha.21-x86_64-apple-darwin.tar.gz"
      sha256 "9f18ddb962b376997b85afcf4c5ed1b85994ee806fdd51cd01fd063b138fce38"
    end
  end

  on_linux do
    url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.21/invar-v0.1.0-alpha.21-x86_64-unknown-linux-musl.tar.gz"
    sha256 "ba522d09c1acde3a7875114c733071f1f695eaec3006db360e1ffe9099330c7b"
  end

  def install
    bin.install "invar"
    generate_completions_from_executable(bin/"invar", "completions") if build.stable?
  end

  test do
    assert_match "invar", shell_output("#{bin}/invar --version")
    # The demo needs a clone of the repo; the formula test stays offline.
    assert_match "cross-repository", shell_output("#{bin}/invar")
  end
end
