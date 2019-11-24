# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0]
### Added
* Version group apis: macros and utilities to defined version maps, version groups and version uris.

### Changed
* The `VersionMap` is now used as a `&dyn VersionMap` and not a generic parameter in types.

## [0.3.1]
### Changed
* To define the version for the current type, use `self` attribute instead of `type = "path to self"`
* Use the [type_name](https://doc.rust-lang.org/std/intrinsics/fn.type_name.html) of a type as
  the key to find its version.

## [0.3.0]
### Added
* Added changelog
* The `v` attribute is an alias for the `version` attribute.
* Added the versioned group pattern with example

### Changed
* The `version` attribute now requires an explicit index.
* The `versions` attribute now requires an entry for the current version.
* Use a trait for VersionMap instead of a HashMap<String, usize>

### Removed
* The `#[versions("Av1", "Av2")]` syntax is not supported anymore, instead use the more explicit version
  `#[versions(v(index = 1, type = "Av1"), v(index = 2, type = "Av2"))]`

## [0.2.3] - 2019-09-08
### Added
- Added an `InvalidVersionError` when the version number provided is not handled by current code.

## [0.2.2] - 2019-09-08
### Added
- Added proper README file

## [0.1.0] - 2019-09-06
### Added
- Versioning feature for serde with additional trait `DeserializeVersioned` and deserializer `VersionedDeserialized`

[Unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/0.4.0...HEAD
[0.4.0]: https://github.com/fredpointzero/serde-version/releases/tag/0.4.0
[0.3.1]: https://github.com/fredpointzero/serde-version/releases/tag/0.3.1
[0.3.0]: https://github.com/fredpointzero/serde-version/releases/tag/0.3.0
[0.2.3]: https://github.com/fredpointzero/serde-version/releases/tag/0.2.3
[0.2.2]: https://github.com/fredpointzero/serde-version/releases/tag/0.2.2
[0.1.0]: https://github.com/fredpointzero/serde-version/releases/tag/0.1.0