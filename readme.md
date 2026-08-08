AUREK - AUR Security Checker

[![Crates.io](https://img.shields.io/crates/v/aurek.svg)](https://crates.io/crates/aurek)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Hack Club Stardance](https://img.shields.io/badge/Hack%20Club-Stardance-blueviolet)](https://stardance.hackclub.com/)
[![Status](https://img.shields.io/badge/status-active-brightgreen.svg)](https://github.com/Erox-02/aurek)

AUREK is a security wrapper for yay that checks AUR packages before installation.

AUR packages are community-maintained and not vetted like official repository packages. A PKGBUILD can execute arbitrary commands during installation. AUREK adds a verification step before that happens.

v2.0.0 - Local LLM analysis with Gemma and llama.cpp is now supported.

---

Quick Start

bash
# Standard security check
aurek -S archey3 --check

# With LLM analysis
aurek -S archey3 --check --model ~/models/gemma-4-E2B-it-Q4_K_M.gguf


---

Features

· PKGBUILD security scanning
· Optional local LLM analysis with Gemma
· yay wrapper for package management commands
· paru detection and warnings
· Colored terminal output
· Git fallback when AUR API is unavailable
· Written in Rust

---
![Screenshot](assets/screenshot.jpg)
---

Installation

From Source

bash
git clone https://github.com/Erox-02/aurek.git
cd aurek
cargo install --path .


Manual Build

bash
git clone https://github.com/Erox-02/aurek.git
cd aurek
cargo build --release
sudo cp target/release/aurek /usr/local/bin/


---

Usage

Check a Package Before Installation

bash
aurek -S <package> --check


Fetches the PKGBUILD, scans for suspicious patterns, displays results, and passes to yay if confirmed.

With LLM Analysis

bash
aurek -S <package> --check --model /path/to/model.gguf


Skip Scan

bash
aurek -S <package> --check --no-scan


Quiet Mode

bash
aurek -S <package> --check --quiet


Normal yay Commands

bash
aurek -Syu
aurek -R <package>
aurek -Q
aurek -Ss <search>


---

Example Output

Safe Package

text
$ aurek -S archey3 --check

AUREK - AUR Security Checker
Checking packages before installation...

Scanning archey3
PKGBUILD saved to /tmp/.tmpF6Xz4F

No suspicious patterns detected.
Executing: yay -S archey3


Suspicious Package

text
$ aurek -S suspicious-package --check

AUREK - AUR Security Checker
Checking packages before installation...

Scanning suspicious-package
PKGBUILD saved to /tmp/.tmpG7Yz5G

Potential issues detected:
  - Remote Code Execution: Downloads script from external URL
  - Privilege Escalation: Uses sudo without verification

Continue anyway? (y/N): n

Installation cancelled.


---

LLM Analysis

Starting with v2.0.0, AUREK can optionally use a local Gemma model through llama.cpp.

Why Local

· Privacy - PKGBUILD stays on local machine
· Offline capability
· Context-aware analysis beyond pattern matching

Setup

Install llama.cpp:

bash
yay -S llama.cpp
# or
sudo pacman -S llama.cpp


Download a Gemma GGUF model:

bash
mkdir -p ~/models
cd ~/models
wget https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF/resolve/main/gemma-4-E2B-it-Q4_K_M.gguf


Run with LLM:

bash
aurek -S package --check --model ~/models/gemma-4-E2B-it-Q4_K_M.gguf


Example with LLM

text
Scanning archey3
PKGBUILD saved to /tmp/.tmpXYZ

Running local LLM analysis with Gemma...
Starting llama-server...
Waiting for server to be ready...
Server started!

No malware detected - Safe to proceed.

llama.cpp server stopped
Executing: yay -S archey3


Heuristics vs LLM

 Heuristics Local LLM
Speed Fast Slower
Analysis Pattern-based Context-aware
Privacy Local Local
Setup Simple Requires model

---

Security Checks

Heuristic Detection

· Remote script execution (curl | sh, wget | sh)
· sudo usage
· Overly permissive permissions (chmod 777)
· Dangerous deletion commands
· Python -c execution
· Base64 decoding
· eval usage
· Systemd service modification
· Cron job installation
· External URLs and network calls

LLM Analysis

Context-aware analysis that can identify suspicious patterns not covered by simple heuristics.

---

Architecture

text
             User
               |
               v
             AUREK
               |
        Is --check enabled?
          +----+----+
         No         Yes
         |           |
         |      Fetch PKGBUILD
         |       API / Git
         |           |
         |           v
         |    Heuristic Scan
         |           |
         |           v
         |    Optional LLM Scan
         |           |
         |           v
         |     Show Findings
         |           |
         +-----+-----+
               |
               v
              yay
               |
               v
        Package Installation


---

Roadmap

Done - v2.0.0

· Basic security scanning
· yay wrapper
· Git fallback
· paru detection
· Local LLM integration
· Colored terminal output

Planned

· Additional LLM models
· More package managers
· Advanced heuristics
· Recursive dependency scanning
· Package reputation system
· Plugin system

---

Limitations

AUREK is a security layer, not a guarantee.

· Static analysis has limits
· Obfuscated code may be missed
· Dependencies can introduce threats
· Runtime behavior is not detected
· False positives and negatives are possible

Manual review of suspicious packages is still recommended.

---

Development

bash
git clone https://github.com/Erox-02/aurek
cd aurek

cargo build
cargo test
cargo build --release
cargo install --path .


---

Contributing

Contributions are welcome.

Areas of contribution:

· Security checks
· Heuristics
· Tests
· LLM integration
· Documentation

Open an issue for larger changes before implementation.

---

License

MIT License. See LICENSE for details.

---

Links

· GitHub: https://github.com/Erox-02/aurek
· Author: https://github.com/Erox-02
