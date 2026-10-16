//! The logic of keepclip that does not depend on the operating system: it
//! reads what yt-dlp reports, turns it into a menu, builds the arguments of the
//! download and follows its progress. Running programs, the network and the
//! console are left to the app.

pub mod checksum;
pub mod config;
pub mod download;
pub mod info;
pub mod link;
pub mod menu;
pub mod progress;
pub mod units;
