# Destination Registry Maintenance

## Purpose

`originweave-destination` contains reviewed address-allocation and cloud-platform endpoint tables that affect SSRF, credential exposure, and redirect safety. They are production security policy, not static examples.

## Cadence

A maintainer or the bounded hourly OpenCode product-development loop must check the sources below **before every release and at least monthly**. The check is fail-closed: a release cannot proceed when the source revision, last-updated date, or table comparison is unknown.

## Authoritative sources

- IANA IPv4 Special-Purpose Address Space
- IANA IPv6 Special-Purpose Address Space
- IANA IPv6 Global Unicast Address Space
- RFC 9637 IPv6 documentation space
- Microsoft Learn documentation for Azure platform IP `168.63.129.16`
- Amazon EKS documentation for Pod Identity endpoints `169.254.170.23` and `fd00:ec2::23`
- immutable Chromium URL canonicalizer regression source at commit `446d05d21720f0b3505ec21057b3e9f909784262`

## Required procedure

1. Record each source's current revision or last-updated date.
2. Compare every production prefix and platform endpoint with the source.
3. Confirm that globally reachable exceptions remain explicit and that unallocated IPv6 global-unicast gaps fail closed.
4. Verify the exact RFC 9637 `3fff::/20` boundaries.
5. Add or update exact lower-bound, upper-bound, adjacent-range, IPv4-mapped, metadata, and resolver-limit regression cases.
6. Update `docs/doctoring.md`, ADR 0004 when semantics change, and `CHANGELOG.md`.
7. Run formatting, locked checks, all tests, strict Clippy, rustdoc, Security Scan, Semgrep, and exact 100% production function, line, region, and branch coverage.
8. Merge only after current-head review threads and required checks are clean.

## Automation boundary

The hourly product-development workflow may open one bounded maintenance PR when drift is detected. It cannot alter tables on `main`, approve its own PR, merge, tag, or release. Organization-level review and merge automation remains independent.
