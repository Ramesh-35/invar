# Homebrew formula for INVAR. Lives here as the source of truth; the
# founder copies it into Ramesh-35/homebrew-tap as Formula/invar.rb.
# Install: brew install Ramesh-35/tap/invar
# Hashes below were re-downloaded and measured on 2026-10-13 (alpha.28), not
# copied from the release page. Regenerate for each release: download
# the three tarballs, sha256sum them, update url/sha256/version.
class Invar < Formula
  desc "Checks cross-repository contracts; a skip is never a pass"
  homepage "https://github.com/Ramesh-35/invar"
  version "0.1.0-alpha.28"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.28/invar-v0.1.0-alpha.28-aarch64-apple-darwin.tar.gz"
      sha256 "34c1cbfe33cca15fc39c03d9538c6201bade732b86d253b33c70a082287c41fe"
    else
      url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.28/invar-v0.1.0-alpha.28-x86_64-apple-darwin.tar.gz"
      sha256 "dff6cbd60b99172255cd0458b5f2f49545ba529bc2ac813c26e023864dcff5e6"
    end
  end

  on_linux do
    url "https://github.com/Ramesh-35/invar/releases/download/v0.1.0-alpha.28/invar-v0.1.0-alpha.28-x86_64-unknown-linux-musl.tar.gz"
    sha256 "04425fdcc8f4bfae5019cc83054dccf6a6b52008748af0535e9b6617529ed640"
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
