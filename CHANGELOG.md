# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- System notifications for upcoming episode releases. [6124e88](https://github.com/MaarifaMaarifa/series-troxide/commit/6124e88fa7b79216ea3c3312bac39910c37746d9)
- `Discover page` refresh using F5 key. [ff543bf](https://github.com/MaarifaMaarifa/series-troxide/commit/ff543bffbcfe04e35a3f3ec037380ff0c6ab6e83)
- Auto-refresh for the `My Shows page` to get accurate episode release time. [8206d82](https://github.com/MaarifaMaarifa/series-troxide/commit/8206d8214d63d2bbb7d6ac9dbb80f4b40a6a058a)
- Average time required to complete watching remaining episodes for a series in `Watchlist page`. [1ee6a8c](https://github.com/MaarifaMaarifa/series-troxide/commit/1ee6a8ca660154a7a2416c26d6b36cf3b6b5b1b6)
- Monthly Airing Series(for new and returning series) sections in `Discover page`. [03d56b5](https://github.com/MaarifaMaarifa/series-troxide/commit/03d56b50869c58cf6e4c9f18536172b88d9af03a)
- Network and web-channel sections(The CW, Netflix, HBO etc) in `Discover page`. [b8ad79b](https://github.com/MaarifaMaarifa/series-troxide/commit/b8ad79bfba3e41ca393049c33913c7964d835e5b)
- Genre sections(Family, Action, Sci-Fi etc) in `Discover page`. [1f2e9b4](https://github.com/MaarifaMaarifa/series-troxide/commit/1f2e9b4b741290226c87ef54329c6fce0f16aa33)
- Suggested Shows(Based on the one currently opened) in `Series page`. [32f8b90](https://github.com/MaarifaMaarifa/series-troxide/commit/32f8b90657fa6319be34b736c693f90e5ca06482)
- Implement proper export file header(magic) with it's version information. [b4518c2](https://github.com/MaarifaMaarifa/series-troxide/commit/b4518c2d2433fe7e51f9271ffe3601dece94c43d)
- More information for each cast in `Series page`. [1c27c8c](https://github.com/MaarifaMaarifa/series-troxide/commit/1c27c8c9f1a14abb3615c4eec27ce8c742cc3750) 

### Changed

- Improve startup speed by preventing all tabs from being loaded. [dedf926](https://github.com/MaarifaMaarifa/series-troxide/commit/dedf92652820a53393d84a7d8cc02380e8af69ee)
- Improve rating widget in `Series page`. [554a5ce](https://github.com/MaarifaMaarifa/series-troxide/commit/554a5ce9b8107dc8f4191926b49bd43e2510a817)
- Improve release time widget in `Series page`. [5abe06c](https://github.com/MaarifaMaarifa/series-troxide/commit/5abe06c4933cca685d8e717d65456b554fedc6b5)
- Improve `My Shows page` loading speed after clicking it's tab. [d7dc366](https://github.com/MaarifaMaarifa/series-troxide/commit/d7dc366f249c4f3f03ada3e3af9c9fd6dc4b5602)
- Improve `Discover Page` loading speed. [0192235](https://github.com/MaarifaMaarifa/series-troxide/commit/01922357e76e2810ff33e5f165e1c14e310036da)
- Redesign `Statistics page`. [7f21393](https://github.com/MaarifaMaarifa/series-troxide/commit/7f21393c54fe40b952cb6501ec2119fd15a88095)
- Move program's cache to the platform-specific cache path. [86e7c92](https://github.com/MaarifaMaarifa/series-troxide/commit/86e7c92e80a90b1d06bb776599b15875476c1efd)
- Arrange the watchlist items shows alphabetically. [51de954](https://github.com/MaarifaMaarifa/series-troxide/commit/51de954e11939d27ed0de59613428a34deace170)
- Improve country selection widget in `Settings page`. [4af4e66](https://github.com/MaarifaMaarifa/series-troxide/commit/4af4e66c5fd16872c7736c4ff9c3fa486b95252b)
- Improve series searching response when loading images. [42c824d](https://github.com/MaarifaMaarifa/series-troxide/commit/42c824d55b3daafd4f21bf691dbcf1fba341b01d)
- Improve speed when getting upcoming episode releases. [ea31d51](https://github.com/MaarifaMaarifa/series-troxide/commit/ea31d51acfc05f4cdc9b7fab63b0c798685ca788)
- Improve cache cleaning. Cache expiration will now be determined using the filesystem. [be4141d](https://github.com/MaarifaMaarifa/series-troxide/commit/be4141d1ab239fc89074af4143a60c13cbe0398d)
- Made casts section expandable in `Series page`. [6a8aa36](https://github.com/MaarifaMaarifa/series-troxide/commit/6a8aa36d5c25e57baad915de62a72092e458aa75)

### Removed

- Shows updates sections in `Discover page`. [08b4b0d](https://github.com/MaarifaMaarifa/series-troxide/commit/08b4b0d6c41c4587ad8c6c0ecf28d57b557c9f88)

### Fixed

- Crash in `My Shows page` when reaching the time for an episode release. [1d1a25e](https://github.com/MaarifaMaarifa/series-troxide/commit/1d1a25ed12a3489ea926c08225568a3944f93da2)
- Duplicate Series Posters in `Discover page`. [a73a9f3](https://github.com/MaarifaMaarifa/series-troxide/commit/a73a9f33ae0c1d3d9f8679bbe78792de168a8730)

## [0.2.0] - 2023-07-27

### Added

- Automatic and manual cache cleaning.
- Tracking data export and import.
- Country selection for locally aired series.

### Changed

- Improve `Series page`.
- Overall UI improvements.

## [0.1.0] - 2023-07-14

### Added

- First release!🎉