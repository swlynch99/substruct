# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.1.1
### Fixed
- Fixed errors on inner `#[substruct]` attributes in cases where the outer
  `#[substruct]` macro returns an error immediately.

## 0.1.0
This is the initial release of substruct.

### Added
- The `#[substruct]` attribute macro
- Support for using `#[substruct_attr]` within to conditionally apply attributes
  to generated substructs.
