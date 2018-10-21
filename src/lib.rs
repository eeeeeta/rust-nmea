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
extern crate hashmap_core;

pub mod time;
mod parse;
#[cfg(test)]
mod test;

use core::{fmt, str, mem};
use alloc::vec::Vec;
use alloc::prelude::*;
use core::iter::Iterator;
use hashmap_core::{HashMap, HashSet};
use time::{NaiveTime, NaiveDate};

pub use parse::{GsvData, GgaData, RmcData, RmcStatusOfFix, parse, ParseResult, GsaData, VtgData};


/// NMEA parser
#[derive(Default)]
pub struct Nmea {
    pub fix_time: Option<NaiveTime>,
    pub fix_date: Option<NaiveDate>,
    pub fix_type: Option<FixType>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<f32>,
    pub speed_over_ground: Option<f32>,
    pub true_course: Option<f32>,
    pub num_of_fix_satellites: Option<u32>,
    pub hdop: Option<f32>,
    pub vdop: Option<f32>,
    pub pdop: Option<f32>,
    pub geoid_height: Option<f32>,
    pub satellites: Vec<Satellite>,
    pub fix_satellites_prns: Option<Vec<u32>>,
    satellites_scan: HashMap<GnssType, Vec<Vec<Satellite>>>,
    required_sentences_for_nav: HashSet<SentenceType>,
    last_fix_time: Option<NaiveTime>,
    sentences_for_this_time: HashSet<SentenceType>,
}

impl<'a> Nmea {
    /// Constructs a new `Nmea`.
    /// This struct parses NMEA sentences, including checksum checks and sentence
    /// validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use nmea::Nmea;
    ///
    /// let mut nmea= Nmea::new();
    /// let gga = "$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76";
    /// nmea.parse(gga).unwrap();
    /// println!("{}", nmea);
    /// ```
    pub fn new() -> Nmea {
        // TODO: This looks ugly.
        let mut n = Nmea::default();
        n.satellites_scan.insert(GnssType::Galileo, vec![]);
        n.satellites_scan.insert(GnssType::Gps, vec![]);
        n.satellites_scan.insert(GnssType::Glonass, vec![]);
        n
    }

    /// Constructs a new `Nmea` for navigation purposes.
    ///
    /// # Examples
    ///
    /// ```
    /// use nmea::{Nmea, SentenceType};
    ///
    /// let mut nmea = Nmea::create_for_navigation([SentenceType::RMC, SentenceType::GGA]
    ///                                                .iter()
    ///                                                .map(|v| v.clone())
    ///                                                .collect()).unwrap();
    /// let gga = "$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76";
    /// nmea.parse(gga).unwrap();
    /// println!("{}", nmea);
    /// ```
    pub fn create_for_navigation(required_sentences_for_nav: HashSet<SentenceType>)
                                 -> Result<Nmea, &'static str> {
        if required_sentences_for_nav.is_empty() {
            return Err("Should be at least one sentence type in required");
        }
        let mut n = Self::new();
        n.required_sentences_for_nav = required_sentences_for_nav;
        Ok(n)
    }


    /// Returns fix type
    pub fn fix_timestamp(&self) -> Option<NaiveTime> {
        self.fix_time
    }

    /// Returns fix type
    pub fn fix_type(&self) -> Option<FixType> {
        self.fix_type.clone()
    }

    /// Returns last fixed latitude in degress. None if not fixed.
    pub fn latitude(&self) -> Option<f64> {
        self.latitude
    }

    /// Returns last fixed longitude in degress. None if not fixed.
    pub fn longitude(&self) -> Option<f64> {
        self.longitude
    }

    /// Returns latitude from last fix. None if not available.
    pub fn altitude(&self) -> Option<f32> {
        self.altitude
    }

    /// Returns the number of satellites use for fix.
    pub fn fix_satellites(&self) -> Option<u32> {
        self.num_of_fix_satellites
    }

    /// Returns the number fix HDOP
    pub fn hdop(&self) -> Option<f32> {
        self.hdop
    }

    /// Returns the height of geoid above WGS84
    pub fn geoid_height(&self) -> Option<f32> {
        self.geoid_height
    }

    /// Returns the height of geoid above WGS84
    pub fn satellites(&self) -> Vec<Satellite> {
        self.satellites.clone()
    }

    fn merge_gga_data(&mut self, gga_data: GgaData) {
        self.fix_time = gga_data.fix_time;
        self.latitude = gga_data.latitude;
        self.longitude = gga_data.longitude;
        self.fix_type = gga_data.fix_type;
        self.num_of_fix_satellites = gga_data.fix_satellites;
        self.hdop = gga_data.hdop;
        self.altitude = gga_data.altitude;
        self.geoid_height = gga_data.geoid_height;
    }

    fn merge_gsv_data(&mut self, data: GsvData) -> Result<(), &'static str> {
        {
            let d = self.satellites_scan
                .get_mut(&data.gnss_type)
                .ok_or("Invalid GNSS type")?;
            // Adjust size to this scan
            d.resize(data.number_of_sentences as usize, vec![]);
            // Replace data at index with new scan data
            d.push(data.sats_info
                       .iter()
                       .filter(|v| v.is_some())
                       .map(|v| v.clone().unwrap())
                       .collect());
            d.swap_remove(data.sentence_num as usize - 1);
        }
        self.satellites.clear();
        for (_, v) in &self.satellites_scan {
            for v1 in v {
                for v2 in v1 {
                    self.satellites.push(v2.clone());
                }
            }
        }

        Ok(())
    }

    fn merge_rmc_data(&mut self, rmc_data: RmcData) {
        self.fix_time = rmc_data.fix_time;
        self.fix_date = rmc_data.fix_date;
        self.fix_type = rmc_data
            .status_of_fix
            .map(|v| match v {
                     RmcStatusOfFix::Autonomous => FixType::Gps,
                     RmcStatusOfFix::Differential => FixType::DGps,
                     RmcStatusOfFix::Invalid => FixType::Invalid,
                 });
        self.latitude = rmc_data.lat;
        self.longitude = rmc_data.lon;
        self.speed_over_ground = rmc_data.speed_over_ground;
        self.true_course = rmc_data.true_course;
    }

    fn merge_gsa_data(&mut self, gsa: GsaData) {
        self.fix_satellites_prns = Some(gsa.fix_sats_prn);
        self.hdop = gsa.hdop;
        self.vdop = gsa.vdop;
        self.pdop = gsa.pdop;
    }

    fn merge_vtg_data(&mut self, vtg: VtgData) {
        self.speed_over_ground = vtg.speed_over_ground;
        self.true_course = vtg.true_course;
    }

    /// Parse any NMEA sentence and stores the result. The type of sentence
    /// is returnd if implemented and valid.
    pub fn parse(&mut self, s: &'a str) -> Result<SentenceType, String> {
        match parse(s.as_bytes())? {
            ParseResult::VTG(vtg) => {
                self.merge_vtg_data(vtg);
                Ok(SentenceType::VTG)
            }
            ParseResult::GGA(gga) => {
                self.merge_gga_data(gga);
                Ok(SentenceType::GGA)
            }
            ParseResult::GSV(gsv) => {
                self.merge_gsv_data(gsv)?;
                Ok(SentenceType::GSV)
            }
            ParseResult::RMC(rmc) => {
                self.merge_rmc_data(rmc);
                Ok(SentenceType::RMC)
            }
            ParseResult::GSA(gsa) => {
                self.merge_gsa_data(gsa);
                Ok(SentenceType::GSA)
            }
            ParseResult::Unsupported(msg_id) => {
                Err(format!("Unknown or implemented sentence type: {:?}", msg_id))
            }
        }
    }

    fn new_tick(&mut self) {
        let old = mem::replace(self, Self::default());
        self.satellites_scan = old.satellites_scan;
        self.satellites = old.satellites;
        self.required_sentences_for_nav = old.required_sentences_for_nav;
        self.last_fix_time = old.last_fix_time;
    }

    fn clear_position_info(&mut self) {
        self.last_fix_time = None;
        self.new_tick();
    }

    pub fn parse_for_fix(&mut self, xs: &[u8]) -> Result<FixType, String> {
        match parse(xs)? {
            ParseResult::GSA(gsa) => {
                self.merge_gsa_data(gsa);
                return Ok(FixType::Invalid);
            }
            ParseResult::GSV(gsv_data) => {
                self.merge_gsv_data(gsv_data)?;
                return Ok(FixType::Invalid);
            }
            ParseResult::VTG(vtg) => {
                //have no time field, so only if user explicity mention it
                if self.required_sentences_for_nav.contains(&SentenceType::VTG) {
                    if vtg.true_course.is_none() || vtg.speed_over_ground.is_none() {
                        self.clear_position_info();
                        return Ok(FixType::Invalid);
                    }
                    self.merge_vtg_data(vtg);
                    self.sentences_for_this_time.insert(SentenceType::VTG);
                } else {
                    return Ok(FixType::Invalid);
                }
            }
            ParseResult::RMC(rmc_data) => {
                match rmc_data.status_of_fix {
                    Some(RmcStatusOfFix::Invalid) |
                    None => {
                        self.clear_position_info();
                        return Ok(FixType::Invalid);
                    }
                    _ => { /*nothing*/ }
                }
                match (self.last_fix_time, rmc_data.fix_time) {
                    (Some(ref last_fix_time), Some(ref rmc_fix_time)) => {
                        if *last_fix_time != *rmc_fix_time {
                            self.new_tick();
                            self.last_fix_time = Some(*rmc_fix_time);
                        }
                    }
                    (None, Some(ref rmc_fix_time)) => self.last_fix_time = Some(*rmc_fix_time),
                    (Some(_), None) | (None, None) => {
                        self.clear_position_info();
                        return Ok(FixType::Invalid);
                    }
                }
                self.merge_rmc_data(rmc_data);
                self.sentences_for_this_time.insert(SentenceType::RMC);
            }
            ParseResult::GGA(gga_data) => {
                match gga_data.fix_type {
                    Some(FixType::Invalid) |
                    None => {
                        self.clear_position_info();
                        return Ok(FixType::Invalid);
                    }
                    _ => { /*nothing*/ }
                }
                match (self.last_fix_time, gga_data.fix_time) {
                    (Some(ref last_fix_time), Some(ref gga_fix_time)) => {
                        if last_fix_time != gga_fix_time {
                            self.new_tick();
                            self.last_fix_time = Some(*gga_fix_time);
                        }
                    }
                    (None, Some(ref gga_fix_time)) => self.last_fix_time = Some(*gga_fix_time),
                    (Some(_), None) | (None, None) => {
                        self.clear_position_info();
                        return Ok(FixType::Invalid);
                    }
                }
                self.merge_gga_data(gga_data);
                self.sentences_for_this_time.insert(SentenceType::GGA);
            }
            ParseResult::Unsupported(_) => {
                return Ok(FixType::Invalid);
            }
        }
        match self.fix_type {
            Some(FixType::Invalid) |
            None => Ok(FixType::Invalid),
            Some(ref fix_type) if self.required_sentences_for_nav
                                      .is_subset(&self.sentences_for_this_time) => {
                Ok(fix_type.clone())
            }
            _ => Ok(FixType::Invalid),
        }
    }
}


impl fmt::Debug for Nmea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Nmea {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}: lat: {} lon: {} alt: {} {:?}",
               self.fix_time
                   .map(|l| format!("{:?}", l))
                   .unwrap_or("None".to_owned()),
               self.latitude
                   .map(|l| format!("{:3.8}", l))
                   .unwrap_or("None".to_owned()),
               self.longitude
                   .map(|l| format!("{:3.8}", l))
                   .unwrap_or("None".to_owned()),
               self.altitude
                   .map(|l| format!("{:.3}", l))
                   .unwrap_or("None".to_owned()),
               self.satellites())
    }
}

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

macro_rules! define_sentence_type_enum {
    ($Name:ident { $($Variant:ident),* $(,)* }) => {
        #[derive(PartialEq, Debug, Hash, Eq, Clone)]
        pub enum $Name {
            None,
            $($Variant),*,
        }

        impl<'a> From<&'a str> for $Name {
            fn from(s: &str) -> Self {
                match s {
                    $(stringify!($Variant) => $Name::$Variant,)*
                    _ => $Name::None,
                }
            }
        }

        impl $Name {
            fn try_from(s: &[u8]) -> Result<Self, &'static str> {
                match str::from_utf8(s).map_err(|_| "invalid header")? {
                    $(stringify!($Variant) => Ok($Name::$Variant),)*
                    _ => Ok($Name::None),
                }
            }
        }
    }
}

#[test]
fn test_define_sentence_type_enum() {
    define_sentence_type_enum!(TestEnum { AAA, BBB });

    let a = TestEnum::AAA;
    let b = TestEnum::BBB;
    let n = TestEnum::None;
    assert_eq!(TestEnum::from("AAA"), a);
    assert_eq!(TestEnum::from("BBB"), b);
    assert_eq!(TestEnum::from("fdafa"), n);

    assert_eq!(TestEnum::try_from(b"AAA").unwrap(), a);
    assert_eq!(TestEnum::try_from(b"BBB").unwrap(), b);
}

/// ! NMEA sentence type
/// ! General: OSD |
/// ! Autopilot: APA | APB | ASD |
/// ! Decca: DCN |
/// ! D-GPS: MSK
/// ! Echo: DBK | DBS | DBT |
/// ! Radio: FSI | SFI | TLL
/// ! Speed: VBW | VHW | VLW |
/// ! GPS: ALM | GBS | GGA | GNS | GSA | GSV |
/// ! Course: DPT | HDG | HDM | HDT | HSC | ROT | VDR |
/// ! Loran-C: GLC | LCD |
/// ! Machine: RPM |
/// ! Navigation: RMA | RMB | RMC |
/// ! Omega: OLN |
/// ! Position: GLL | DTM
/// ! Radar: RSD | TLL | TTM |
/// ! Rudder: RSA |
/// ! Temperature: MTW |
/// ! Transit: GXA | RTF |
/// ! Waypoints and tacks: AAM | BEC | BOD | BWC | BWR | BWW | ROO | RTE |
/// !                      VTG | WCV | WNC | WPL | XDR | XTE | XTR |
/// ! Wind: MWV | VPW | VWR |
/// ! Date and Time: GDT | ZDA | ZFO | ZTG |
define_sentence_type_enum!(SentenceType {
                               AAM,
                               ABK,
                               ACA,
                               ACK,
                               ACS,
                               AIR,
                               ALM,
                               ALR,
                               APA,
                               APB,
                               ASD,
                               BEC,
                               BOD,
                               BWC,
                               BWR,
                               BWW,
                               CUR,
                               DBK,
                               DBS,
                               DBT,
                               DCN,
                               DPT,
                               DSC,
                               DSE,
                               DSI,
                               DSR,
                               DTM,
                               FSI,
                               GBS,
                               GGA,
                               GLC,
                               GLL,
                               GMP,
                               GNS,
                               GRS,
                               GSA,
                               GST,
                               GSV,
                               GTD,
                               GXA,
                               HDG,
                               HDM,
                               HDT,
                               HMR,
                               HMS,
                               HSC,
                               HTC,
                               HTD,
                               LCD,
                               LRF,
                               LRI,
                               LR1,
                               LR2,
                               LR3,
                               MLA,
                               MSK,
                               MSS,
                               MWD,
                               MTW,
                               MWV,
                               OLN,
                               OSD,
                               ROO,
                               RMA,
                               RMB,
                               RMC,
                               ROT,
                               RPM,
                               RSA,
                               RSD,
                               RTE,
                               SFI,
                               SSD,
                               STN,
                               TLB,
                               TLL,
                               TRF,
                               TTM,
                               TUT,
                               TXT,
                               VBW,
                               VDM,
                               VDO,
                               VDR,
                               VHW,
                               VLW,
                               VPW,
                               VSD,
                               VTG,
                               VWR,
                               WCV,
                               WNC,
                               WPL,
                               XDR,
                               XTE,
                               XTR,
                               ZDA,
                               ZDL,
                               ZFO,
                               ZTG,
                           });

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

