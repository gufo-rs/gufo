# Changelog

## 0.5.5 (2026-10-26)

### Fixed

- exif: Fix wrong offset in data for lookups in some images where the remaining data were read from a wrong position.

## 0.5.4 (2026-10-19)

### Fixed

- exif: Fix panic during initial camera manufacturer lookup

## 0.5.3 (2026-10-13)

### Fixed

- Remove unnecessary dbg! statements

## 0.5.2 (2026-10-13)

### Fixed

- exif: Don't use unwrap in `ParserGeneric::read_bytes` #15
- exif: Only parse canon-style MakerNote if Make is "Canon". This fixes parse errors on Exif from camera makers with different MakerNote structure.
- png: Fix TransferCharacteristics test

## 0.5.1 (2026-09-08)

### Fixed

- ImageMetadata::xmp() returning Exif data for JPEGs

## 0.5.0-alpha (2026-06-07)

### Added

- New `gufo-svg` crate to extract metadata from SVGs.

### Changed

- Complete rewrite of the `gufo-exif` crate with completely new API.
