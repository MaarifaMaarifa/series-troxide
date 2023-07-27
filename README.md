# Series Troxide

A Simple and Modern Series Tracker.

![](screenshots/discover-page.png)

## Features
- **Aired and New Series discovery**. See what's new globally and locally.
- **Series search**. Search for your favourite Series.
- **Upcoming releases**. See when your tracked series are being aired.
- **Series Information**. See general information of any series (Summary, genres, casts, etc).
- **Series Categorization**. See which of your series are running, ended and untracked.
- **Series watch progress tracking**. See what season and episode you left off and how many are unwatched.
- **Series Statistics**. See how many series, seasons and episodes you have watched and how much time you've spent watching them in an ordered way.
- **Light and Dark themes**. Use **Series Troxide** at any time of the day.
- **Database export and import**. Carry your series tracking data anywhere.
- **Caching**. Due to the rate limit of the API, caching makes **Series Troxide** fast when opening previously opened items and be able to perform crazy things like getting the statistics of all watched series. Cache cleaning can be managed both automatically and manually to make sure the program does not have outdated series data.
- **Notifications for upcoming episodes**. _in progress_.

## Installation

### Getting pre-built binaries
Pre-built binaries for your specific platform can be obtained from the [release page](https://github.com/MaarifaMaarifa/series-troxide/releases)

### Cargo ([crates.io](https://crates.io/crates/series-troxide))
**Series Troxide** is available in crates.io and can be installed using Cargo.
```shell
cargo install series-troxide
```
### Building from source.
You can build **Series Troxide** from source assuming you have Git, Cargo and Rustc set up on your machine. You can check the [guide](https://rustup.rs/) incase you're not setup.
```shell
git clone https://github.com/MaarifaMaarifa/series-troxide
cd series-troxide
cargo install --path .
```

## Credits
- The API used has been provided by TVmaze, you can check out the site [here](https://www.tvmaze.com/).
- The Icons used have been provided by boostrap icons, you can check out the site [here](https://icons.getbootstrap.com/).
- The Graphical User Interface has been made using Iced, you can check out the site [here](https://iced.rs/).
- The UI design has been inspired by the [Hobi Mobile app](https://hobiapp.com/)
