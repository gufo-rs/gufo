#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub lat: Coord,
    pub lon: Coord,
}

impl Location {
    pub fn new_from_coord(lat: Coord, lon: Coord) -> Self {
        Self { lat, lon }
    }

    pub fn from_ref_coord(
        lat_ref: LatRef,
        lat: (f64, f64, f64),
        lon_ref: LonRef,
        lon: (f64, f64, f64),
    ) -> Self {
        let lat = Coord::from_sign_deg_min_sec(lat_ref.as_sign(), lat);
        let lon = Coord::from_sign_deg_min_sec(lon_ref.as_sign(), lon);

        Self { lat, lon }
    }

    pub fn lat_ref_deg_min_sec(&self) -> (LatRef, (f64, f64, f64)) {
        let (deg, min, sec) = self.lat.as_deg_min_sec();
        (LatRef::from_sign(deg), (deg.abs(), min, sec))
    }

    pub fn lon_ref_deg_min_sec(&self) -> (LonRef, (f64, f64, f64)) {
        let (deg, min, sec) = self.lon.as_deg_min_sec();
        (LonRef::from_sign(deg), (deg.abs(), min, sec))
    }

    /// Return coordinate according to ISO 6709 Annex D
    ///
    /// <https://en.wikipedia.org/wiki/ISO_6709>
    ///
    /// ```
    /// # use gufo_common::geography::*;
    /// let lat = Coord::from_deg_min_sec((-46., 14., 6.));
    /// let lon = Coord::from_deg_min_sec((126., 4., 6.70234));
    /// let loc = Location::new_from_coord(lat, lon);
    /// assert_eq!(loc.iso_6709(), r#"46°14'06"S 126°04'06.7"E"#);
    pub fn iso_6709(&self) -> String {
        let (lat_ref, (lat_deg, lat_min, lat_sec)) = self.lat_ref_deg_min_sec();
        let (lon_ref, (lon_deg, lon_min, lon_sec)) = self.lon_ref_deg_min_sec();

        fn pad_one_0(v: f64) -> String {
            let s = format!("{v}");

            let pre_decimal = s.split_once('.').map_or(s.as_str(), |x| x.0);

            if pre_decimal.len() == 1 {
                format!("0{s}")
            } else {
                s
            }
        }

        let lat_sec = pad_one_0(lat_sec);
        let lon_sec = pad_one_0(lon_sec);

        format!("{lat_deg}°{lat_min:02}'{lat_sec}\"{lat_ref} {lon_deg}°{lon_min:02}'{lon_sec}\"{lon_ref}")
    }

    /// Locations as `geo:` URI
    ///
    /// The precision of the coordinates is limited to six decimal places.
    pub fn geo_uri(&self) -> String {
        let lat = self.lat.0;
        let lon = self.lon.0;
        // six decimal places gives as more than a meter accuracy
        format!("geo:{lat:.6},{lon:.6}")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Coord(pub f64);

impl Coord {
    /// Return coordinate as degrees, minutes, seconds
    ///
    /// ```
    /// # use gufo_common::geography::*;
    /// let ang = Coord::from_deg_min_sec((-46., 14., 6.70));
    /// assert_eq!(ang.as_deg_min_sec(), (-46., 14., 6.70));
    /// ```
    pub fn as_deg_min_sec(&self) -> (f64, f64, f64) {
        let deg = self.0;
        let h = deg.fract().abs() * 60.;
        let s = h.fract() * 60.;

        (deg.trunc(), h.trunc(), (s * 100.).round() / 100.)
    }

    ///
    ///
    /// ```
    /// # use gufo_common::geography::*;
    /// let ang = Coord::from_deg_min_sec((-89., 24., 2.2));
    /// assert_eq!((ang.0 * 100_000.).round() / 100_000., -89.40061);
    /// ```
    pub fn from_deg_min_sec((deg, min, sec): (f64, f64, f64)) -> Self {
        let sign = deg.signum();
        Coord(deg + sign * min / 60. + sign * sec / 60. / 60.)
    }

    ///
    ///
    /// ```
    /// # use gufo_common::geography::*;
    /// let ang = Coord::from_sign_deg_min_sec(LatRef::South.as_sign(), (89., 24., 2.2));
    /// assert_eq!((ang.0 * 100_000.).round() / 100_000., -89.40061);
    /// ```
    pub fn from_sign_deg_min_sec(sign: f64, deg_min_sec: (f64, f64, f64)) -> Self {
        Self(sign * Self::from_deg_min_sec(deg_min_sec).0)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LatRef {
    North,
    South,
}

impl LatRef {
    pub fn from_sign(sign: f64) -> Self {
        if sign >= 0. {
            Self::North
        } else {
            Self::South
        }
    }

    pub fn as_sign(&self) -> f64 {
        match self {
            Self::North => 1.,
            Self::South => -1.,
        }
    }
}

impl TryFrom<&str> for LatRef {
    type Error = InvalidLatRef;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "N" => Ok(Self::North),
            "S" => Ok(Self::South),
            v => Err(Self::Error::InvalidLatitudeRef(v.to_string())),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidLatRef {
    #[error("Invalid latitude reference: '{0}'. Must be 'N' or 'S'.")]
    InvalidLatitudeRef(String),
}

impl std::fmt::Display for LatRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::North => f.write_str("N"),
            Self::South => f.write_str("S"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LonRef {
    East,
    West,
}

impl LonRef {
    pub fn from_sign(sign: f64) -> Self {
        if sign >= 0. {
            Self::East
        } else {
            Self::West
        }
    }

    pub fn as_sign(&self) -> f64 {
        match self {
            Self::East => 1.,
            Self::West => -1.,
        }
    }
}

impl TryFrom<&str> for LonRef {
    type Error = InvalidLonRef;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "E" => Ok(Self::East),
            "W" => Ok(Self::West),
            v => Err(Self::Error::InvalidLonRef(v.to_string())),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum InvalidLonRef {
    #[error("Invalid latitude reference: '{0}'. Must be 'E' or 'W'.")]
    InvalidLonRef(String),
}

impl std::fmt::Display for LonRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::East => f.write_str("E"),
            Self::West => f.write_str("W"),
        }
    }
}
