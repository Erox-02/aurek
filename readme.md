# AUREK - AUR Security Checker

> Check your AUR packages before they check your system.

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Hack Club Stardance](https://img.shields.io/badge/Hack%20Club-Stardance-blueviolet)](https://stardance.hackclub.com/)
[![Status](https://img.shields.io/badge/status-active-brightgreen.svg)](https://github.com/Erox-02/aurek)

AUREK is a security wrapper around `yay` that checks AUR packages before installing them.

The AUR is awesome, but packages there are maintained by the community and aren't vetted the same way packages in the official Arch repositories are. A `PKGBUILD` can run commands on your machine while building a package, so blindly installing random AUR packages isn't always a great idea.

AUREK adds another checkpoint.

Before installation, it can fetch the package's `PKGBUILD`, scan it for suspicious patterns, show you anything weird it finds, and then let you decide whether you still want to continue.

Basically:

```text
yay install package
package maybe evil
computer sad

aurek check package
aurek see suspicious thing
human decide
computer hopefully less sad
```

It's written in Rust because this is a security tool and I wanted something fast, reliable, and memory-safe.

AUREK is still being developed, so don't treat it as a replacement for actually reading a `PKGBUILD`.

## Features

* Security scanning for suspicious `PKGBUILD` patterns
* Wrapper around `yay`
* Passes normal `yay` commands through
* Detects `paru` and warns about unsupported usage
* Colored and readable terminal output
* Git fallback if the normal AUR fetching method fails
* Fast and lightweight Rust binary
* Designed so more package managers can be supported later

## Why AUREK?

The Arch User Repository has a huge amount of useful software.

The problem is that AUR packages are community maintained.

When you install an AUR package, you're trusting its build instructions. A malicious or compromised `PKGBUILD` could potentially execute commands you really don't want running on your machine.

Most people probably aren't going to manually inspect every line of every `PKGBUILD`.

Me included.

So I started building AUREK.

It doesn't magically make the AUR safe, and it definitely can't guarantee that a package isn't malicious. It just adds another layer between:

```text
"cool package"
```

and

```text
"run random internet code on my computer"
```

## Installation

### From crates.io

Once published:

```bash
cargo install aurek
```

### From source

```bash
git clone https://github.com/Erox-02/aurek.git
cd aurek
cargo install --path .
```

### Build manually

```bash
git clone https://github.com/Erox-02/aurek.git
cd aurek

cargo build --release
```

The compiled binary will be at:

```text
target/release/aurek
```

You can then move it somewhere in your `PATH`.

For example:

```bash
sudo cp target/release/aurek /usr/local/bin/aurek
```

## Usage

### Install a package with a security check

```bash
aurek -S <package> --check
```

Example:

```bash
aurek -S archey3 --check
```

### Normal yay passthrough

Without `--check`, AUREK behaves mostly like a wrapper around `yay`.

```bash
aurek -S <package>
```

Other `yay` commands can also be passed through:

```bash
aurek -Syu
aurek -R <package>
aurek -Q
aurek -Ss <search>
```

### Skip scanning

```bash
aurek -S <package> --check --no-scan
```

### Quiet mode

```bash
aurek -S <package> --check --quiet
```

## Example

```text
$ aurek -S archey3 --check

AUREK - AUR Security Checker
Checking packages before installation...

Scanning archey3
PKGBUILD saved to /tmp/.tmpF6Xz4F

No suspicious patterns detected.

Executing:
yay -S archey3
```

If something suspicious is found:

```text
$ aurek -S suspicious-package --check

AUREK - AUR Security Checker
Checking packages before installation...

Scanning suspicious-package
PKGBUILD saved to /tmp/.tmpG7Yz5G

Potential issues detected:

- Uses sudo, may escalate privileges
- External URL detected: https://example.com/script.sh

Continue anyway? (y/N): n

Installation cancelled.
```

A warning doesn't automatically mean a package is malware.

There are completely legitimate reasons for some packages to use commands that look suspicious. AUREK's job is to point them out so you can actually look at them before continuing.

AUREK see weird command. AUREK tell human. Human use brain.

## How It Works

When you run something like:

```bash
aurek -S package-name --check
```

AUREK roughly does this:

1. Detects the package being installed.
2. Fetches its `PKGBUILD` from the AUR.
3. Falls back to Git if the normal fetching method fails.
4. Runs heuristic security checks against the `PKGBUILD`.
5. Reports suspicious patterns.
6. Asks for confirmation when necessary.
7. Passes the installation command to `yay`.

So AUREK isn't replacing `yay`.

It sits in front of it.

```text
You
 |
 v
AUREK
 |
 +---- fetch PKGBUILD
 |
 +---- scan PKGBUILD
 |
 +---- suspicious?
 |        |
 |        +---- yes ---> warn human
 |        |
 |        +---- no
 |
 v
yay
 |
 v
package installation
```

## Security Checks

AUREK currently looks for patterns related to things such as:

* Remote script execution
* `curl` or `wget` piped into a shell
* `sudo` usage
* Suspicious privilege escalation
* Overly permissive permissions such as `chmod 777`
* Dangerous deletion commands
* Python `-c` execution
* Base64 decoding and possible obfuscation
* Shell `eval`
* Systemd service modifications
* Cron job installation
* External URLs
* Unexpected network calls

These checks are heuristic.

That means AUREK can produce false positives and it can also miss malicious code.

Do not use:

```text
AUREK says safe = definitely safe
```

Use:

```text
AUREK found nothing obvious = still use brain
```

Security hard.

Computer do exactly what code say.

Sometimes code say bad thing.

## LLM Integration

One of the main things I want to add is optional local LLM analysis.

The idea is to run a small language model locally, such as Gemma through `llama.cpp`, and give it the `PKGBUILD` alongside the normal heuristic results.

Instead of only matching patterns like:

```bash
curl something | sh
```

an LLM could potentially look at the context around commands and identify suspicious behavior that simple pattern matching might miss.

The important part is **local**.

I don't want AUREK uploading random `PKGBUILD` files or analysis data to some external AI API just to check a package.

The planned flow looks something like:

```text
PKGBUILD
   |
   +------> Heuristic Scanner
   |
   +------> Local LLM
                 |
                 v
          Security Analysis
                 |
                 v
             AUREK
```

This is still experimental and not finished yet.

LLM smart sometimes.

LLM also confidently dumb sometimes.

So it will be another signal, not the final authority.

## Roadmap

### LLM Integration

Add optional local language model analysis using models such as Gemma through `llama.cpp`.

### More Package Managers

AUREK currently focuses on `yay`, but I want the scanning system to be less dependent on one AUR helper.

Possible future support includes other AUR helpers and eventually other package ecosystems.

### Advanced Heuristics

Improve the scanner beyond basic pattern matching.

This could include:

* Better shell command analysis
* Obfuscation detection
* Suspicious download detection
* Build-step analysis
* File modification analysis
* Better severity scoring

### Recursive Dependency Checking

Scan relevant dependencies instead of checking only the package directly requested by the user.

### Package Reputation

Possibly create a community-driven reputation system for packages and detected threats.

This needs careful design because "random internet people vote package evil" is not exactly a perfect security system.

### Plugin System

Allow custom scanners and security rules.

### Web Interface

Eventually, I'd like to build a web interface for viewing package analysis reports and possibly a community threat database.

For now though:

terminal good.

terminal fast.

web later.

## Project Status

AUREK is currently under active development.

Some things will change.

Some things will break.

Some code will probably make me wonder why I wrote it like that three weeks later.

That's normal.

The current focus is getting the basic scanning and `yay` integration reliable before adding more complicated detection systems.

## Limitations

AUREK is an extra security layer, not an antivirus and not a guarantee that an AUR package is safe.

Static analysis has limits.

For example, malicious behavior could be:

* Hidden in downloaded source code
* Introduced after the initial scan
* Obfuscated in a way the scanner doesn't understand
* Executed through dependencies
* Triggered at runtime rather than build time

Always use common sense when installing software from untrusted sources.

If something looks extremely weird, read the `PKGBUILD`.

## Development

Clone the repository:

```bash
git clone https://github.com/Erox-02/aurek.git
cd aurek
```

Build:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Build an optimized release:

```bash
cargo build --release
```

Install locally:

```bash
cargo install --path .
```

## Contributing

Contributions are welcome.

If you want to help, you can:

* Report bugs
* Suggest security checks
* Improve existing heuristics
* Work on LLM integration
* Add tests
* Improve documentation
* Help support additional package managers

Open an issue if you have an idea before making a huge change so we don't accidentally build two completely different versions of the same thing.

Small PR good.

Working code good.

Tests very good.

## Security

If you discover a security problem in AUREK itself, please avoid publicly posting exploit details before there's a chance to fix the issue.

Security tool having security vulnerability = very not good.

## License

AUREK is licensed under the MIT License.

See the `LICENSE` file for details.

## Acknowledgments

Thanks to:

* Hack Club Stardance for inspiring the project
* The `yay` developers and contributors
* The Arch Linux and AUR communities
* The Rust community and ecosystem

## Links

GitHub:

[https://github.com/Erox-02/aurek](https://github.com/Erox-02/aurek)

