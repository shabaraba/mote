# Changelog

## [0.2.1](https://github.com/shabaraba/mote/compare/v0.2.0...v0.2.1) (2026-01-28)


### Code Refactoring

* redesign CLI with unified command structure ([#21](https://github.com/shabaraba/mote/issues/21)) ([ad5d5ec](https://github.com/shabaraba/mote/commit/ad5d5ecc30c8bab75fc852ac590f4ffca4db875d))


### Documentation

* add comprehensive project/context management documentation ([#17](https://github.com/shabaraba/mote/issues/17)) ([59a2204](https://github.com/shabaraba/mote/commit/59a220400757778df5615183031a586c4e5c6290)), closes [#16](https://github.com/shabaraba/mote/issues/16)

## [0.2.0](https://github.com/shabaraba/mote/compare/v0.1.2...v0.2.0) (2026-01-26)


### ⚠ BREAKING CHANGES

* --storage-dir option removed, use --context-dir instead

### Features

* add configurable ignore file path for mote init ([#13](https://github.com/shabaraba/mote/issues/13)) ([ab6ec82](https://github.com/shabaraba/mote/commit/ab6ec8268009266f49b5ebb950cdb5ea21c7693b))
* implement 3-layer project/context management system ([#15](https://github.com/shabaraba/mote/issues/15)) ([00501e3](https://github.com/shabaraba/mote/commit/00501e3d0fb4ffe3baea4cd66e03db6b2190b39b))

## [0.1.2](https://github.com/shabaraba/mote/compare/v0.1.1...v0.1.2) (2026-01-20)


### Bug Fixes

* configure release workflows to trigger properly ([#5](https://github.com/shabaraba/mote/issues/5)) ([6857643](https://github.com/shabaraba/mote/commit/685764312ce9b1057b2f5f1bd9f8199abe3de309))
* correct Homebrew formula URL pattern to include 'v' prefix ([#8](https://github.com/shabaraba/mote/issues/8)) ([90333fe](https://github.com/shabaraba/mote/commit/90333fe36a9cb852574a486cec4648e2a400df7c))
* update Cargo.toml version and change release-type to rust ([#10](https://github.com/shabaraba/mote/issues/10)) ([85ab4a8](https://github.com/shabaraba/mote/commit/85ab4a89ed2a2c16820a740b34417b4db8c7fc3d))
* update SHA256 checksums to match actual v0.1.1 release assets ([#9](https://github.com/shabaraba/mote/issues/9)) ([a610680](https://github.com/shabaraba/mote/commit/a610680387f17bbfe72159e66eb991f7a5149c40))
* use RELEASE_PLEASE_TOKEN for homebrew-tap access ([#7](https://github.com/shabaraba/mote/issues/7)) ([6f30a6b](https://github.com/shabaraba/mote/commit/6f30a6b9b457021d93f372aa9ba31e9ad53a5956))


### Documentation

* Improve OSS repository structure and documentation ([#12](https://github.com/shabaraba/mote/issues/12)) ([99b4e41](https://github.com/shabaraba/mote/commit/99b4e41c28560e6f299a36ade2834ce50d089c87))

## [0.1.1](https://github.com/shabaraba/mote/compare/v0.1.0...v0.1.1) (2026-01-19)


### Features

* Add CLI options for custom paths (project-root, ignore-file, storage-dir) ([#4](https://github.com/shabaraba/mote/issues/4)) ([faecc62](https://github.com/shabaraba/mote/commit/faecc62cf5586a5b79187e71cf359c80655fd651))
* add git/jj shell integration and refactor codebase ([#1](https://github.com/shabaraba/mote/issues/1)) ([0f095b5](https://github.com/shabaraba/mote/commit/0f095b52b99f81899776609936a0de5ed9632f9f))
* Initial implementation of mote snapshot tool ([82519f0](https://github.com/shabaraba/mote/commit/82519f0a7a2292e6080206a781059a2937c2c594))
