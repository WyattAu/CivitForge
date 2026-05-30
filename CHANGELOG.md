# Changelog

All notable changes to the CivitForge project are documented in this file.

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-05-30

### Added
- Established project specification foundation (Phase 0: Requirements Engineering)
- Created domain analysis with 7 applicable standards mapped (ISO 27001, NIST SP 800-53, SLSA L4, FIPS 140-2, ISO 26262, OWASP Top 10, SOC2 Type II)
- Converted 21 PRD functional requirements into 69 EARS-format requirements across 5 areas (VCS, LFS+, CI/CD, AI, Federation)
- Added 16 formalized non-functional requirements with measurable acceptance criteria
- Created traceability matrix mapping all 69 requirements to components, test cases, standards, and roadmap phases
- Identified 6 standard conflicts with proposed resolution strategies (SLSA vs air-gap, rootless vs performance, consistency vs FINRA, AI sandbox vs utility, FIPS vs Rust crypto, VFS vs air-gap)
- Generated capability matrix mapping available tooling against required stack
- Identified 10 tooling gaps requiring procurement or installation before development

### Project Structure
- `.specs/00_requirements/` -- Requirements engineering artifacts
- `.reports/` -- Phase reports and analysis documents
