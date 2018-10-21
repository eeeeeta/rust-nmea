//! NMEA 0183 parser
//!
//! Use nmea::Nmea::parse and nmea::Nmea::parse_for_fix to preserve
//! state between recieving new nmea sentence, and nmea::parse
//! to parse sentences without state
//!
//! Units that used every where: degrees, knots, meters for altitude
#![feature(alloc)]
// Copyright (C) 2016 Felix Obenhuber
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

#![no_std]
#[cfg(test)]
extern crate quickcheck;
#[macro_use]
extern crate nom;
#[cfg(test)]
#[macro_use]
extern crate approx;
#[macro_use]
extern crate alloc;

pub mod time;
pub mod parse;
#[cfg(test)]
mod test;

use core::fmt;
use alloc::prelude::*;

pub use parse::{GsvData, GgaData, RmcData, RmcStatusOfFix, parse, ParseResult, GsaData, VtgData};

#[derive(Clone, PartialEq)]
/// ! A Satellite
pub struct Satellite {
    gnss_type: GnssType,
    prn: u32,
    elevation: Option<f32>,
    azimuth: Option<f32>,
    snr: Option<f32>,
}

impl Satellite {
    pub fn gnss_type(&self) -> GnssType {
        self.gnss_type.clone()
    }

    pub fn prn(&self) -> u32 {
        self.prn
    }

    pub fn elevation(&self) -> Option<f32> {
        self.elevation
    }

    pub fn azimuth(&self) -> Option<f32> {
        self.azimuth
    }

    pub fn snr(&self) -> Option<f32> {
        self.snr
    }
}

impl fmt::Display for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: {} elv: {} ath: {} snr: {}",
               self.gnss_type,
               self.prn,
               self.elevation
                   .map(|e| format!("{}", e))
                   .unwrap_or("--".to_owned()),
               self.azimuth
                   .map(|e| format!("{}", e))
                   .unwrap_or("--".to_owned()),
               self.snr
                   .map(|e| format!("{}", e))
                   .unwrap_or("--".to_owned()))
    }
}

impl fmt::Debug for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "[{:?},{:?},{:?},{:?},{:?}]",
               self.gnss_type,
               self.prn,
               self.elevation,
               self.azimuth,
               self.snr)
    }
}
/// ! Fix type
#[derive(Clone, PartialEq, Debug)]
pub enum FixType {
    Invalid,
    Gps,
    DGps,
    Pps,
    Rtk,
    FloatRtk,
    Estimated,
    Manual,
    Simulation,
}

/// ! GNSS type
#[derive (Debug, Clone, Hash, Eq, PartialEq)]
pub enum GnssType {
    Galileo,
    Gps,
    Glonass,
}

impl fmt::Display for GnssType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GnssType::Galileo => write!(f, "Galileo"),
            GnssType::Gps => write!(f, "GPS"),
            GnssType::Glonass => write!(f, "GLONASS"),
        }
    }
}

impl From<char> for FixType {
    fn from(x: char) -> Self {
        match x {
            '0' => FixType::Invalid,
            '1' => FixType::Gps,
            '2' => FixType::DGps,
            '3' => FixType::Pps,
            '4' => FixType::Rtk,
            '5' => FixType::FloatRtk,
            '6' => FixType::Estimated,
            '7' => FixType::Manual,
            '8' => FixType::Simulation,
            _ => FixType::Invalid,
        }
    }
}

