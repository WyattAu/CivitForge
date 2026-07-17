# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in CivitForge, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

### How to Report

Email: security@civitforge.dev (or use GitHub's private vulnerability reporting feature)

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

| Stage | Timeline |
|---|---|
| Acknowledgment | Within 24 hours |
| Initial assessment | Within 72 hours |
| Fix or mitigation | Within 7 days for critical; 30 days for others |

### What We Fix

- Remote code execution
- Authentication/authorization bypass
- Data injection (SQL, command, etc.)
- Path traversal
- Privilege escalation
- Cryptographic weaknesses
- Denial of service in critical paths

### What We Don't Fix (as security issues)

- Denial of service in non-critical paths
- Social engineering
- Issues in third-party dependencies (report upstream)
- Theoretical attacks with no practical exploit

## Safe Harbor

We support safe harbor for security researchers who:

- Make a good faith effort to avoid privacy violations and data destruction
- Only interact with accounts you own or with explicit permission
- Do not exploit a vulnerability beyond what is necessary to confirm its existence
- Report promptly and do not disclose publicly before a fix is available

We will not pursue legal action against researchers who follow these guidelines.

## Scope

This policy covers the CivitForge codebase at `github.com/WyattAu/CivitForge`.

## Disclosure

We request 90 days from initial report before public disclosure. We will credit reporters in the release notes unless they prefer anonymity.
