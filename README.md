
![GitHub release (latest)](https://img.shields.io/github/v/release/cicero-ai/nyx)
![License](https://img.shields.io/github/license/cicero-ai/nyx)
![Build](https://img.shields.io/github/actions/workflow/status/cicero-ai/nyx/ci.yml)
![Downloads](https://img.shields.io/github/downloads/cicero-ai/nyx/total)

# Nyx

Secure command line utility to manage passwords, authenticator app OTP codes, SSH keys, notes, and protect sensitive credential files.

## Features

* Simple non-interactive CLI command with time based locking of database based on inactivity.
* Seamless category support with forward slashes (ie. nyx new mysite/cloudflare, nyx ls mysite)
* Passwords always securely available, instantly copied to clipboard (ie. nyx xp mysite/cloudflare)
* Create authenticator app entry with Base32 secret, instantly generate 6 digit OTP auth codes (ie. nyx otp site-name)
* Built-in ssh-agent plus functionality to easily and securely import, generate and manage SSH keys. (Linux and Mac only)
* Create and manage notes with default text editor (vi, namo, etc.) (ie. nyx note new some-title)
* Secure sensitive credentials files behind a fuse point, allowing only specified binaries read access.
* AES-GCM, Argon2, hkdf, auto-clearing of clipboard every 120 seconds.
* Aargon2  parameters: memory=64MB, iterations=2, parallelism=4


Simplistic, out of the way, yet always accessible and just works.


## Installation

Download Binary: https://github.com/cicero-ai/nyx/releases/tag/v1.1.0

cargo (Rust - installs 'nyx' binary):
```bash
cargo install nyxpass
```

Homebrew (no fuse support):
```bash
brew tap cicero-ai/homebrew-tools
brew install nyxpass
```


**Mac Users:** To enable fuse point with protected files, you must install [MacFUSE](https://macfuse.github.io/) v10.9 or later.  
If using Apple Silicon, you must also enable support for third party kernel extensions.
Pre-compiled Mac binaries do not have fuse support.  If installing via cargo with MacFUSE, run: 
```bash
cargo install --features fuse
```


## Quick Start

Check installation: 
    `nyx --version`


No setup required, you'll be prompted to create database during first write.  Looks for database files in this order:

* -f or --dbfile CLI flags
* NYX_DBFILE environment variable
* ~/.local/share/nyx/nyx.db
* Prompts for location


**Common Commands**

Data Type | Action | Example
----------| ----------| ----------
Help | Home | `nyx help`
&nbsp; | Category | `nyx help ssh`
&nbsp; | Command | `nyx help ssh import`
User | Create | `nyx new mysite/cloudflare`
&nbsp; | List | `nyx ls` / `nyx ls mysite`
&nbsp; | Copy Password | `nyx xp mysite/cloudflare`
OTP* | Create | `nyx otp new namecheap`
&nbsp; | Generate 6 Digit OTP | `nyx otp namecheap`
SSH Key** | Import | `nyx ssh import mysite/server1 --file /path/to/server1.pem`
&nbsp; | Generate New | `nyx ssh generate mysite/server2`
&nbsp; | Copy Public Key | `nyx ssh xb mysite/server2`
String | Set | `nyx set mysite/stripe-api-secret "SK:live:123"`
&nbsp; | Get / Copy | `nyx get mysite/stripe-api-secret`
Notes | Create | `nyx note new mysite/long-secrets`
&nbsp; | Edit | `nyx note edit mysite/long-secrets`
&nbsp; | Display | `nyx note show mysite/long-secrets`
&nbsp; | Copy to Clipboard | `nyx note xn mysite/long-secrets`
Database | Close | `nyx close`
&nbsp; | Change Password | `nyx db changepass`
&nbsp; | Backup | `nyx backup`
&nbsp; | History Log | `nyx db history`

Files | Protect | `nyx protect config.yaml`
&nbsp; | Freeze | `nyx freeze config.yaml 5`
&nbsp; | Restore | `nyx restore config.yaml`
&nbsp; | Scan All | `nyx scan`

**NOTE:**  All data types (User, OTP, SSH, String, Note) share the same core commands (create, update, delete, copy, rename, etc.). Use `nyx help <CATEGORY>` for a full list of available commands.


###File Protection

Nyx helps protect against supply chain attacks by putting your sensitive credential files behind a fuse point.  Your credentials will b stored within the encrypted volume, and only available to the whitelisted binaries you specify (eg. only "aws" binary can read the AWS credential files).

If a binary outside of the whitelist tries to open a protected file, a desktop notification is sent plus a log added to ~/nyx_unauthorized_access.log file in your home directory.

Scan your machine for known credential files and move them under protection with the command: `nyx scan`

**NOTE:** This is only available to Linux and Mac users.  For Linux, it works out of the box, while Mac users need to ensure MacFuse is installed.


## About the Developer

Nyx is developed by [Aquila Labs](https://aquila-labs.ca/), a Canadian firm that prides itself on quality,  self hosted, privacy focused software.

Cicero is the main and ongoing project: https://cicero.sh/r/manifesto


