# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Added
- Full MCP JSON-RPC 2.0 server with stdio transport
- Capability-based access control (6 groups, all off by default)
- Tamper-evident audit log with blake3 hash chain
- 22 MCP tools across Generate, Inject, Swarm, Profile, Query, System groups
- 4 MCP resources (presets, categories, statistics, capabilities)
- 3 MCP prompt templates (quick_start, journalist_protection, maximum_protection)
- Inject and Swarm capabilities require typed acknowledgment
- Resource limits (artifacts/min, injection bytes, swarm storage, bandwidth, concurrent tasks)
- 43 tests (24 unit + 18 integration + 1 doc-test)
- CLI with clap (--transport, --data-dir flags)
- CLAUDE.md with disambiguation between infrastructure repos and PlausiDenOS components
