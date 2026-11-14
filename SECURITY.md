# Security policy

mumble-client-protocol is part of the Mumcord project. The [Mumcord security policy](https://github.com/Mumcord/mumcord/blob/main/SECURITY.md) applies to it in full: what to expect after a report, disclosure, and the rules for testing.

## Supported versions

Security fixes are made for the **latest release** and ship as a new patch release. Older releases do not get fixes; please update to the latest release.

## Reporting a vulnerability

**Do not report vulnerabilities in public issues, pull requests or discussions.**

Report privately through GitHub:

**<https://github.com/Mumcord/mumble-client-protocol/security/advisories/new>**

Only you and the maintainers can read the report. If you cannot use this form, open a normal issue that only says you would like to report a security problem and asks for a private channel. Do not include any details in it.

A good report contains the crate version, the Mumble server version, what an attacker can achieve and what they need for it (for example: a malicious server sending a crafted message), and steps to reproduce.

## What counts as a vulnerability

- A server or another client that can crash the client, make it hang or use unbounded memory through crafted messages or voice packets.
- A server certificate that is accepted although the caller's check rejected it, or a TLS or voice crypto weakness beyond the ones the Mumble protocol itself has.
- Secrets such as passwords or private keys ending up in logs.

Weaknesses of the Mumble protocol itself (for example its OCB2 voice crypto) are best reported to the [Mumble project](https://github.com/mumble-voip/mumble); tell us too if the crate can mitigate them.
