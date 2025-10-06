
![GitHub release (latest)](https://img.shields.io/github/v/release/cicero-ai/nyx)
![License](https://img.shields.io/github/license/cicero-ai/nyx)
![Build](https://img.shields.io/github/actions/workflow/status/cicero-ai/nyx/ci.yml)
![Downloads](https://img.shields.io/github/downloads/cicero-ai/nyx/total)

# Nyx - Secure Password, OTP and SSH Keys Management

Secure command line utility to manage passwords, authenticator app OTP codes, SSH keys, and notes.

## Features

* Simple non-interactive CLI command with time based locking of database based on inactivity.
* Seamless category support with forward slashes (ie. 'nyx new mysite/cloudflare', 'nyx ls mysite')
* Passwords always securely available, instantly copied to clipboard (ie. `nyx xp mysite/cloudflare`, `nyx xu mysite/cloudflare`)
* Create authenticator app entry with Base32 secret, instantly generate 6 digit OTP auth codes (ie. `nyx otp site-name`)
* SSH keys available via virtual fuse point filesystem (Linux / Mac only).  Import SSH keys, modify IdentityFile parameter in ~/.ssh/config file to point to /tmp/nyx/ssh_keys/<NAME>.
* Create and manage notes with default text editor (vi, namo, etc.) (ie. `nyx note new some-title`)
* AES-GCM, Argon2, hkdf, auto-clearing of clipboard every 120 seconds.
* Supports multiple databases and localhost RPC API.

Simplistic, out of the way, yet always accessible and just works.


## Installation

Download Binary: https://github.com/cicero-ai/nyx/releases/tag/v1.0.0

Rust / Cargo:  cargo install nyxpass  (installs 'nyx' binary)

Homebrew:  [coming]

**Mac Users:** To enable fuse point with SSH keys, you must install [MacFUSE](https://macfuse.github.io/) v10.9 or later.  
If using Apple Silicon, you must also enable support for third party kernel extensions.
If installing via cargo without MacFUSE, run: cargo install --no-default-features


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

### Additional Notes

**OTP Codes:** When registering an authenticator app, you'll be provided a QR code 
and a Base32 secret. Use the Base32 secret when creating a new OTP entry in Nyx.

**SSH Keys (Linux/Mac only):** Nyx mounts a FUSE filesystem at `/tmp/nyx/ssh_keys/` 
when you open your database. Update your `~/.ssh/config` IdentityFile paths to 
point to `/tmp/nyx/ssh_keys/<NAME>` to keep keys encrypted while maintaining your 
normal SSH workflow.

* All data types (User, OTP, SSH, String, Note) share the same core commands (create, update, delete, copy, rename, etc.). Use `nyx help <CATEGORY>` for a full list of available commands.


## Stay Updated

For the latest on Nyx, you can always view the latest and subscribe to the mailing list at: [https://cicero.sh/nyx](https://cicero.sh/nyx)

## Related Project

If you found this software helpful, check out [Cicero](https://cicero.sh/latest) - a self hosted AI assistant 
focused on protecting our personal privacy in the age of AI.
    [https://cicero.sh/latest](https://cicero.sh/latest)


