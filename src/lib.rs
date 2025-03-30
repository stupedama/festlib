//! # festlib
//!
//! This project provides a set of functions to parse data from the fest xml file.
//! fest xml file contains data from Norwegian Medical Products Agency (DMP) for
//!
//! This project is licensed under Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT).
//!
//! # Examples
//!
//! ### Date fest file was updated
//! ```
//! use festlib::Fest;
//!
//! let fest = Fest::new("fest251.xml").unwrap();
//! let date = fest.delivery_date();
//!
//! assert_eq!(date.date(), "2024-09-09T14:21:28");
//! ```
//!
//! ### Find packages
//! ```
//! use festlib::Fest;
//!
//! let fest = Fest::new("fest251.xml").unwrap();
//! let packages = fest.packages();
//!
//! assert_eq!(packages.len(), 10473);
//! ```
//!
//! ### Find generic packages
//! ```
//! use festlib::Fest;
//! let fest = Fest::new("fest251.xml").unwrap();
//! let package = fest.find_package("061561").unwrap();
//!
//! let result = fest.find_generic(&package);
//! ```
//! # Contact
//! For questions or feedback use make a issue on our github or john.doe.hemmelig@pm.me.
//!

mod fest;
mod xml;
mod types;

pub use crate::fest::Fest;
pub use crate::types::Package;
