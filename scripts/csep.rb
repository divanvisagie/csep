class Csep < Formula
  desc "Terminal client for Chat GPT"
  homepage "https://github.com/divanvisagie/csep"
  
  # Dynamically set the URL and SHA256 based on the CPU architecture
  if Hardware::CPU.intel?
    url "https://github.com/divanvisagie/csep/releases/download/{{tag}}/csep-darwin-x86_64.tar.gz"
    sha256 "{{intel_hash}}"
  elsif Hardware::CPU.arm?
    url "https://github.com/divanvisagie/csep/releases/download/{{tag}}/csep-darwin-aarch64.tar.gz"
    sha256 "{{arm_hash}}"
  else
    odie "Your architecture is not supported!"
  end

  def install
    bin.install "csep"
    man1.install "csep.1"
  end

  test do
    system "#{bin}/csep --version"
  end
end
