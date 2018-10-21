
#[test]
fn test_fix_type() {
    assert_eq!(FixType::from('A'), FixType::Invalid);
    assert_eq!(FixType::from('0'), FixType::Invalid);
    assert_eq!(FixType::from('1'), FixType::Gps);
    assert_eq!(FixType::from('2'), FixType::DGps);
    assert_eq!(FixType::from('3'), FixType::Pps);
    assert_eq!(FixType::from('4'), FixType::Rtk);
    assert_eq!(FixType::from('5'), FixType::FloatRtk);
    assert_eq!(FixType::from('6'), FixType::Estimated);
    assert_eq!(FixType::from('7'), FixType::Manual);
    assert_eq!(FixType::from('8'), FixType::Simulation);
}

#[test]
fn test_checksum() {
    use parse::checksum;
    let valid = "$GNGSA,A,1,,,,,,,,,,,,,99.99,99.99,99.99*2E";
    let invalid = "$GNZDA,165118.00,13,05,2016,00,00*71";
    assert_eq!(checksum((&valid[1..valid.len() - 3]).as_bytes().iter()),
               0x2E);
    assert_ne!(checksum((&invalid[1..invalid.len() - 3]).as_bytes().iter()),
               0x71);
}

#[test]
fn test_message_type() {
    assert_eq!(SentenceType::try_from(b"GGA").unwrap(), SentenceType::GGA);
    assert_eq!(SentenceType::try_from(b"XXX").unwrap(), SentenceType::None);
}

#[test]
fn test_gga_north_west() {
    use chrono::Timelike;
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76")
        .unwrap();
    assert_eq!(nmea.fix_timestamp().unwrap().second(), 50);
    assert_eq!(nmea.fix_timestamp().unwrap().minute(), 27);
    assert_eq!(nmea.fix_timestamp().unwrap().hour(), 9);
    assert_eq!(nmea.latitude().unwrap(), 53. + 21.6802 / 60.);
    assert_eq!(nmea.longitude().unwrap(), -(6. + 30.3372 / 60.));
    assert_eq!(nmea.fix_type().unwrap(), FixType::Gps);
    assert_eq!(nmea.fix_satellites().unwrap(), 8);
    assert_eq!(nmea.hdop().unwrap(), 1.03);
    assert_eq!(nmea.geoid_height().unwrap(), 55.2);
}

#[test]
fn test_gga_north_east() {
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,N,00630.3372,E,1,8,1.03,61.7,M,55.2,M,,*64")
        .unwrap();
    assert_eq!(nmea.latitude().unwrap(), 53. + 21.6802 / 60.);
    assert_eq!(nmea.longitude().unwrap(), 6. + 30.3372 / 60.);
}

#[test]
fn test_gga_south_west() {
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,S,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*6B")
        .unwrap();
    assert_eq!(nmea.latitude().unwrap(), -(53. + 21.6802 / 60.));
    assert_eq!(nmea.longitude().unwrap(), -(6. + 30.3372 / 60.));
}

#[test]
fn test_gga_south_east() {
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,S,00630.3372,E,1,8,1.03,61.7,M,55.2,M,,*79")
        .unwrap();
    assert_eq!(nmea.latitude().unwrap(), -(53. + 21.6802 / 60.));
    assert_eq!(nmea.longitude().unwrap(), 6. + 30.3372 / 60.);
}

#[test]
fn test_gga_invalid() {
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,S,00630.3372,E,0,8,1.03,61.7,M,55.2,M,,*7B")
        .unwrap_err();
    assert_eq!(nmea.fix_type(), None);
}

#[test]
fn test_gga_gps() {
    use chrono::Timelike;
    let mut nmea = Nmea::new();
    nmea.parse("$GPGGA,092750.000,5321.6802,S,00630.3372,E,1,8,1.03,61.7,M,55.2,M,,*79")
        .unwrap();
    assert_eq!(nmea.fix_timestamp().unwrap().second(), 50);
    assert_eq!(nmea.fix_timestamp().unwrap().minute(), 27);
    assert_eq!(nmea.fix_timestamp().unwrap().hour(), 9);
    assert_eq!(-(53. + 21.6802 / 60.), nmea.latitude.unwrap());
    assert_eq!(6. + 30.3372 / 60., nmea.longitude.unwrap());
    assert_eq!(nmea.fix_type(), Some(FixType::Gps));
    assert_eq!(8, nmea.num_of_fix_satellites.unwrap());
    assert_eq!(1.03, nmea.hdop.unwrap());
    assert_eq!(61.7, nmea.altitude.unwrap());
    assert_eq!(55.2, nmea.geoid_height.unwrap());
}

#[test]
fn test_gsv() {
    let mut nmea = Nmea::new();
    //                        10           07           05           08
    nmea.parse("$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70")
        .unwrap();
    //                        02           13           26         04
    nmea.parse("$GPGSV,3,2,11,02,39,223,19,13,28,070,17,26,23,252,,04,14,186,14*79")
        .unwrap();
    //                        29           16         36
    nmea.parse("$GPGSV,3,3,11,29,09,301,24,16,09,020,,36,,,*76")
        .unwrap();
    assert_eq!(nmea.satellites().len(), 11);

    let sat: &Satellite = &(nmea.satellites()[0]);
    assert_eq!(sat.gnss_type, GnssType::Gps);
    assert_eq!(sat.prn, 10);
    assert_eq!(sat.elevation, Some(63.0));
    assert_eq!(sat.azimuth, Some(137.0));
    assert_eq!(sat.snr, Some(17.0));
}

#[test]
fn test_gsv_real_data() {
    let mut nmea = Nmea::new();
    let real_data = ["$GPGSV,3,1,12,01,49,196,41,03,71,278,32,06,02,323,27,11,21,196,39*72",
                     "$GPGSV,3,2,12,14,39,063,33,17,21,292,30,19,20,310,31,22,82,181,36*73",
                     "$GPGSV,3,3,12,23,34,232,42,25,11,045,33,31,45,092,38,32,14,061,39*75",
                     "$GLGSV,3,1,10,74,40,078,43,66,23,275,31,82,10,347,36,73,15,015,38*6B",
                     "$GLGSV,3,2,10,75,19,135,36,65,76,333,31,88,32,233,33,81,40,302,38*6A",
                     "$GLGSV,3,3,10,72,40,075,43,87,00,000,*6F",

                     "$GPGSV,4,4,15,26,02,112,,31,45,071,,32,01,066,*4C"];
    for line in &real_data {
        assert_eq!(nmea.parse(line).unwrap(), SentenceType::GSV);
    }
}

#[test]
fn test_gsv_order() {
    let mut nmea = Nmea::new();
    //                         2           13           26         04
    nmea.parse("$GPGSV,3,2,11,02,39,223,19,13,28,070,17,26,23,252,,04,14,186,14*79")
        .unwrap();
    //                        29           16         36
    nmea.parse("$GPGSV,3,3,11,29,09,301,24,16,09,020,,36,,,*76")
        .unwrap();
    //                        10           07           05           08
    nmea.parse("$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70")
        .unwrap();
    assert_eq!(nmea.satellites().len(), 11);

    let sat: &Satellite = &(nmea.satellites()[0]);
    assert_eq!(sat.gnss_type, GnssType::Gps);
    assert_eq!(sat.prn, 10);
    assert_eq!(sat.elevation, Some(63.0));
    assert_eq!(sat.azimuth, Some(137.0));
    assert_eq!(sat.snr, Some(17.0));
}

#[test]
fn test_gsv_two_of_three() {
    let mut nmea = Nmea::new();
    //                         2           13           26          4
    nmea.parse("$GPGSV,3,2,11,02,39,223,19,13,28,070,17,26,23,252,,04,14,186,14*79")
        .unwrap();
    //                        29           16         36
    nmea.parse("$GPGSV,3,3,11,29,09,301,24,16,09,020,,36,,,*76")
        .unwrap();
    assert_eq!(nmea.satellites().len(), 7);
}

#[test]
fn test_parse() {
    let sentences = ["$GPGGA,092750.000,5321.6802,N,00630.3372,W,1,8,1.03,61.7,M,55.2,M,,*76",
                     "$GPGSA,A,3,10,07,05,02,29,04,08,13,,,,,1.72,1.03,1.38*0A",
                     "$GPGSV,3,1,11,10,63,137,17,07,61,098,15,05,59,290,20,08,54,157,30*70",
                     "$GPGSV,3,2,11,02,39,223,19,13,28,070,17,26,23,252,,04,14,186,14*79",
                     "$GPGSV,3,3,11,29,09,301,24,16,09,020,,36,,,*76",
                     "$GPRMC,092750.000,A,5321.6802,N,00630.3372,W,0.02,31.66,280511,,,A*43"];

    let mut nmea = Nmea::new();
    for s in &sentences {
        let res = nmea.parse(s).unwrap();
        println!("test_parse res {:?}", res);
    }

    assert_eq!(nmea.latitude().unwrap(), 53. + 21.6802 / 60.);
    assert_eq!(nmea.longitude().unwrap(), -(6. + 30.3372 / 60.));
    assert_eq!(nmea.altitude().unwrap(), 61.7);
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::QuickCheck;
    use super::parse::checksum;

    fn check_parsing_lat_lon_in_gga(lat: f64, lon: f64) -> bool {
        let lat_min = (lat.abs() * 60.0) % 60.0;
        let lon_min = (lon.abs() * 60.0) % 60.0;
        let mut nmea = Nmea::new();
        let mut s = format!("$GPGGA,092750.000,{lat_deg:02}{lat_min:09.6},{lat_dir},\
                             {lon_deg:03}{lon_min:09.6},{lon_dir},1,8,1.03,61.7,M,55.2,M,,*",
                            lat_deg = lat.abs().floor() as u8, lon_deg = lon.abs().floor() as u8,
                            lat_min = lat_min, lon_min = lon_min,
                            lat_dir = if lat.is_sign_positive() { 'N' } else { 'S' },
                            lon_dir = if lon.is_sign_positive() { 'E' } else { 'W' },
        );
        let cs = checksum(s.as_bytes()[1..s.len() - 1].iter());
        s.push_str(&format!("{:02X}", cs));
        nmea.parse(&s).unwrap();
        let (new_lat, new_lon) = (nmea.latitude.unwrap(), nmea.longitude.unwrap());
        const MAX_COOR_DIFF: f64 = 1e-7;
        (new_lat - lat).abs() < MAX_COOR_DIFF && (new_lon - lon).abs() < MAX_COOR_DIFF
    }

    #[test]
    fn test_parsing_lat_lon_in_gga() {
        // regressions found by quickcheck,
        // explicit because of quickcheck use random gen
        assert!(check_parsing_lat_lon_in_gga(0., 57.89528));
        assert!(check_parsing_lat_lon_in_gga(0., -43.33031));
        QuickCheck::new()
            .tests(10_000_000_000)
            .quickcheck(check_parsing_lat_lon_in_gga as fn(f64, f64) -> bool);
    }
}

#[test]
fn test_parse_for_fix() {
    {
        let mut nmea = Nmea::create_for_navigation([SentenceType::RMC, SentenceType::GGA]
                                                       .iter()
                                                       .map(|v| v.clone())
                                                       .collect())
                .unwrap();
        let log = [("$GPRMC,123308.2,A,5521.76474,N,03731.92553,E,000.48,071.9,090317,010.2,E,A*3B",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 200))),
                   ("$GPGGA,123308.2,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*52",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 200))),
                   ("$GPVTG,071.9,T,061.7,M,000.48,N,0000.88,K,A*10",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 200))),
                   ("$GPRMC,123308.3,A,5521.76474,N,03731.92553,E,000.51,071.9,090317,010.2,E,A*32",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 300))),
                   ("$GPGGA,123308.3,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*53",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 300))),
                   ("$GPVTG,071.9,T,061.7,M,000.51,N,0000.94,K,A*15",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 300))),
                   ("$GPRMC,123308.4,A,5521.76474,N,03731.92553,E,000.54,071.9,090317,010.2,E,A*30",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 400))),
                   ("$GPGGA,123308.4,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*54",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 400))),
                   ("$GPVTG,071.9,T,061.7,M,000.54,N,0001.00,K,A*1C",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 400))),
                   ("$GPRMC,123308.5,A,5521.76474,N,03731.92553,E,000.57,071.9,090317,010.2,E,A*32",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 500))),
                   ("$GPGGA,123308.5,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*55",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 500))),
                   ("$GPVTG,071.9,T,061.7,M,000.57,N,0001.05,K,A*1A",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 500))),
                   ("$GPRMC,123308.6,A,5521.76474,N,03731.92553,E,000.58,071.9,090317,010.2,E,A*3E",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 600))),
                   ("$GPGGA,123308.6,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*56",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 600))),
                   ("$GPVTG,071.9,T,061.7,M,000.58,N,0001.08,K,A*18",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 600))),
                   ("$GPRMC,123308.7,A,5521.76474,N,03731.92553,E,000.59,071.9,090317,010.2,E,A*3E",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 700))),
                   ("$GPGGA,123308.7,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*57",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 700))),
                   ("$GPVTG,071.9,T,061.7,M,000.59,N,0001.09,K,A*18",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 700)))];

        for (i, item) in log.iter().enumerate() {
            let res = nmea.parse_for_fix(item.0.as_bytes()).unwrap();
            println!("parse result({}): {:?}, {:?}", i, res, nmea.fix_time);
            assert_eq!((&res, &nmea.fix_time), (&item.1, &item.2));
        }
    }

    {
        let mut nmea = Nmea::create_for_navigation([SentenceType::RMC, SentenceType::GGA]
                                                       .iter()
                                                       .map(|v| v.clone())
                                                       .collect())
                .unwrap();
        let log = [("$GPRMC,123308.2,A,5521.76474,N,03731.92553,E,000.48,071.9,090317,010.2,E,A*3B",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 200))),
                   ("$GPRMC,123308.3,A,5521.76474,N,03731.92553,E,000.51,071.9,090317,010.2,E,A*32",
                    FixType::Invalid,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 300))),
                   ("$GPGGA,123308.3,5521.76474,N,03731.92553,E,1,08,2.2,211.5,M,13.1,M,,*53",
                    FixType::Gps,
                    Some(NaiveTime::from_hms_milli(12, 33, 8, 300)))];

        for (i, item) in log.iter().enumerate() {
            let res = nmea.parse_for_fix(item.0.as_bytes()).unwrap();
            println!("parse result({}): {:?}, {:?}", i, res, nmea.fix_time);
            assert_eq!((&res, &nmea.fix_time), (&item.1, &item.2));
        }
    }
}

#[test]
fn test_some_reciever() {
    let lines = ["$GPRMC,171724.000,A,6847.2474,N,03245.8351,E,0.26,140.74,250317,,*02",
                 "$GPGGA,171725.000,6847.2473,N,03245.8351,E,1,08,1.0,87.7,M,18.5,M,,0000*66",
                 "$GPGSA,A,3,02,25,29,12,31,06,23,14,,,,,2.0,1.0,1.7*3A",
                 "$GPRMC,171725.000,A,6847.2473,N,03245.8351,E,0.15,136.12,250317,,*05",
                 "$GPGGA,171726.000,6847.2473,N,03245.8352,E,1,08,1.0,87.8,M,18.5,M,,0000*69",
                 "$GPGSA,A,3,02,25,29,12,31,06,23,14,,,,,2.0,1.0,1.7*3A",
                 "$GPRMC,171726.000,A,6847.2473,N,03245.8352,E,0.16,103.49,250317,,*0E",
                 "$GPGGA,171727.000,6847.2474,N,03245.8353,E,1,08,1.0,87.9,M,18.5,M,,0000*6F",
                 "$GPGSA,A,3,02,25,29,12,31,06,23,14,,,,,2.0,1.0,1.7*3A",
                 "$GPRMC,171727.000,A,6847.2474,N,03245.8353,E,0.49,42.80,250317,,*32"];
    let mut nmea = Nmea::create_for_navigation([SentenceType::RMC, SentenceType::GGA]
                                                   .iter()
                                                   .map(|v| v.clone())
                                                   .collect())
            .unwrap();
    println!("start test");
    let mut nfixes = 0_usize;
    for line in &lines {
        match nmea.parse_for_fix(line.as_bytes()) {
            Ok(FixType::Invalid) => {
                println!("invalid");
                continue;
            }
            Err(msg) => {
                println!("update_gnss_info_nmea: parse_for_fix failed: {}", msg);
                continue;
            }
            Ok(_) => nfixes += 1,
        }
    }
    assert_eq!(nfixes, 3);
}

#[test]
fn test_parse_rmc() {
    let s = parse_nmea_sentence(b"$GPRMC,225446.33,A,4916.45,N,12311.12,W,\
                                  000.5,054.7,191194,020.3,E,A*2B")
            .unwrap();
    assert_eq!(s.checksum, s.calc_checksum());
    assert_eq!(s.checksum, 0x2b);
    let rmc_data = parse_rmc(&s).unwrap();
    assert_eq!(rmc_data.fix_time.unwrap(), NaiveTime::from_hms_milli(22, 54, 46, 330));
    assert_eq!(rmc_data.fix_date.unwrap(), NaiveDate::from_ymd(94, 11, 19));

    println!("lat: {}", rmc_data.lat.unwrap());
    relative_eq!(rmc_data.lat.unwrap(), 49.0 + 16.45 / 60.);
    println!("lon: {}, diff {}", rmc_data.lon.unwrap(),
             (rmc_data.lon.unwrap() + (123.0 + 11.12 / 60.)).abs());
    relative_eq!(rmc_data.lon.unwrap(), -(123.0 + 11.12 / 60.));

    relative_eq!(rmc_data.speed_over_ground.unwrap(), 0.5);
    relative_eq!(rmc_data.true_course.unwrap(), 54.7);

    let s = parse_nmea_sentence(b"$GPRMC,,V,,,,,,,,,,N*53").unwrap();
    let rmc = parse_rmc(&s).unwrap();
    assert_eq!(RmcData {
        fix_time: None,
        fix_date: None,
        status_of_fix: Some(RmcStatusOfFix::Invalid),
        lat: None,
        lon: None,
        speed_over_ground: None,
        true_course: None,
    }, rmc);
}

#[test]
fn test_float_number() {
    assert_eq!(IResult::Done(&b""[..], &b"12.3"[..]), float_number(&b"12.3"[..]));
    assert_eq!(IResult::Done(&b"a"[..], &b"12.3"[..]), float_number(&b"12.3a"[..]));
    assert_eq!(IResult::Done(&b"a"[..], &b"12"[..]), float_number(&b"12a"[..]));
    assert_eq!(IResult::Error(nom::ErrorKind::Digit), float_number(&b"a12a"[..]));
}

#[test]
fn test_parse_vtg() {
    let run_parse_vtg = |line: &str| -> Result<VtgData, String> {
        let s = parse_nmea_sentence(line.as_bytes()).expect("VTG sentence initial parse failed");
        assert_eq!(s.checksum, s.calc_checksum());
        parse_vtg(&s)
    };
    assert_eq!(VtgData{ true_course: None, speed_over_ground: None },
               run_parse_vtg("$GPVTG,,T,,M,,N,,K,N*2C").unwrap());
    assert_eq!(VtgData{ true_course: Some(360.), speed_over_ground: Some(0.) },
               run_parse_vtg("$GPVTG,360.0,T,348.7,M,000.0,N,000.0,K*43").unwrap());
    assert_eq!(VtgData{ true_course: Some(54.7), speed_over_ground: Some(5.5) },
               run_parse_vtg("$GPVTG,054.7,T,034.4,M,005.5,N,010.2,K*48").unwrap());
}

#[test]
fn test_parse_gsv_full() {
    let data = parse_gsv(&NmeaSentence {
                             talker_id: b"GP",
                             message_id: b"GSV",
                             data: b"2,1,08,01,,083,46,02,17,308,,12,07,344,39,14,22,228,",
                             checksum: 0,
                         })
            .unwrap();
    assert_eq!(data.gnss_type, GnssType::Gps);
    assert_eq!(data.number_of_sentences, 2);
    assert_eq!(data.sentence_num, 1);
    assert_eq!(data._sats_in_view, 8);
    assert_eq!(data.sats_info[0].clone().unwrap(),
               Satellite {
                   gnss_type: data.gnss_type.clone(), prn: 1, elevation: None,
                   azimuth: Some(83.), snr: Some(46.)
               });
    assert_eq!(data.sats_info[1].clone().unwrap(),
               Satellite {
                   gnss_type: data.gnss_type.clone(), prn: 2, elevation: Some(17.),
                   azimuth: Some(308.), snr: None});
    assert_eq!(data.sats_info[2].clone().unwrap(),
               Satellite {
                   gnss_type: data.gnss_type.clone(), prn: 12, elevation: Some(7.),
                   azimuth: Some(344.), snr: Some(39.)});
    assert_eq!(data.sats_info[3].clone().unwrap(),
               Satellite {
                   gnss_type: data.gnss_type.clone(), prn: 14, elevation: Some(22.),
                   azimuth: Some(228.), snr: None});

    let data = parse_gsv(&NmeaSentence {
                             talker_id: b"GL",
                             message_id: b"GSV",
                             data: b"3,3,10,72,40,075,43,87,00,000,",
                             checksum: 0,
                         })
            .unwrap();
    assert_eq!(data.gnss_type, GnssType::Glonass);
    assert_eq!(data.number_of_sentences, 3);
    assert_eq!(data.sentence_num, 3);
    assert_eq!(data._sats_in_view, 10);
}

#[test]
fn test_parse_hms() {
    use chrono::Timelike;
    let (_, time) = parse_hms(b"125619,").unwrap();
    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 56);
    assert_eq!(time.second(), 19);
    assert_eq!(time.nanosecond(), 0);
    let (_, time) = parse_hms(b"125619.5,").unwrap();
    assert_eq!(time.hour(), 12);
    assert_eq!(time.minute(), 56);
    assert_eq!(time.second(), 19);
    assert_eq!(time.nanosecond(), 5_00_000_000);
}


#[test]
fn test_parse_gga_full() {
    let data = parse_gga(&NmeaSentence {
                             talker_id: b"GP",
                             message_id: b"GGA",
                             data: b"033745.0,5650.82344,N,03548.9778,E,1,07,1.8,101.2,M,14.7,M,,",
                             checksum: 0x57,
                         })
            .unwrap();
    assert_eq!(data.fix_time.unwrap(), NaiveTime::from_hms(3, 37, 45));
    assert_eq!(data.fix_type.unwrap(), FixType::Gps);
    relative_eq!(data.latitude.unwrap(), 56. + 50.82344 / 60.);
    relative_eq!(data.longitude.unwrap(), 35. + 48.9778 / 60.);
    assert_eq!(data.fix_satellites.unwrap(), 7);
    relative_eq!(data.hdop.unwrap(), 1.8);
    relative_eq!(data.altitude.unwrap(), 101.2);
    relative_eq!(data.geoid_height.unwrap(), 14.7);

    let s = parse_nmea_sentence(b"$GPGGA,,,,,,0,,,,,,,,*66").unwrap();
    assert_eq!(s.checksum, s.calc_checksum());
    let data = parse_gga(&s).unwrap();
    assert_eq!(GgaData {
        fix_time: None,
        fix_type: Some(FixType::Invalid),
        latitude: None,
        longitude: None,
        fix_satellites: None,
        hdop: None,
        altitude: None,
        geoid_height: None,
    }, data);
}

#[test]
fn test_parse_gga_with_optional_fields() {
    let sentence =
        parse_nmea_sentence(b"$GPGGA,133605.0,5521.75946,N,03731.93769,E,0,00,,,M,,M,,*4F")
            .unwrap();
    assert_eq!(sentence.checksum, sentence.calc_checksum());
    assert_eq!(sentence.checksum, 0x4f);
    let data = parse_gga(&sentence).unwrap();
    assert_eq!(data.fix_type.unwrap(), FixType::Invalid);
}

#[test]
fn test_gsa_prn_fields_parse() {
    let (_, ret) = gsa_prn_fields_parse(b"5,").unwrap();
    assert_eq!(vec![Some(5)], ret);
    let (_, ret) = gsa_prn_fields_parse(b",").unwrap();
    assert_eq!(vec![None], ret);

    let (_, ret) = gsa_prn_fields_parse(b",,5,6,").unwrap();
    assert_eq!(vec![None, None, Some(5), Some(6)], ret);
}

#[test]
fn smoke_test_parse_gsa() {
    let s = parse_nmea_sentence(b"$GPGSA,A,3,,,,,,16,18,,22,24,,,3.6,2.1,2.2*3C").unwrap();
    let gsa = parse_gsa(&s).unwrap();
    assert_eq!(GsaData {
        mode1: GsaMode1::Automatic,
        mode2: GsaMode2::Fix3D,
        fix_sats_prn: vec![16,18,22,24],
        pdop: Some(3.6),
        hdop: Some(2.1),
        vdop: Some(2.2),
    }, gsa);
    let gsa_examples = ["$GPGSA,A,3,19,28,14,18,27,22,31,39,,,,,1.7,1.0,1.3*35",
                        "$GPGSA,A,3,23,31,22,16,03,07,,,,,,,1.8,1.1,1.4*3E",
                        "$BDGSA,A,3,214,,,,,,,,,,,,1.8,1.1,1.4*18",
                        "$GNGSA,A,3,31,26,21,,,,,,,,,,3.77,2.55,2.77*1A",
                        "$GNGSA,A,3,75,86,87,,,,,,,,,,3.77,2.55,2.77*1C",
                        "$GPGSA,A,1,,,,*32"];
    for line in &gsa_examples {
        println!("we parse line '{}'", line);
        let s = parse_nmea_sentence(line.as_bytes()).unwrap();
        parse_gsa(&s).unwrap();
    }
}

