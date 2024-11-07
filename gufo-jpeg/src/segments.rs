use std::io::Read;

use super::Error;

#[derive(Debug)]
pub struct Dqt_<T> {
    tq: u8,
    qk: [T; 64],
}

/// Quantization Table
#[derive(Debug)]
pub enum Dqt {
    Dqt8(Dqt_<u8>),
    Dqt16(Dqt_<u16>),
}

impl Dqt {
    pub fn tq(&self) -> u8 {
        match self {
            Self::Dqt8(x) => x.tq,
            Self::Dqt16(x) => x.tq,
        }
    }

    pub fn qk(&self) -> [u16; 64] {
        match self {
            Self::Dqt8(dqt) => {
                let mut qk = [0; 64];
                for (n, i) in dqt.qk.into_iter().enumerate() {
                    qk[n] = i.into();
                }
                qk
            }
            Self::Dqt16(dqt) => dqt.qk,
        }
    }

    pub fn from_data(mut value: &[u8]) -> Result<Vec<Self>, Error> {
        let mut dqts = Vec::new();
        while !value.is_empty() {
            let mut pq_tq = [0; 1];
            value
                .read_exact(&mut pq_tq)
                .map_err(|_| Error::UnexpectedEof)?;
            let pq_tq = pq_tq[0];

            let pq = pq_tq >> 4;
            let tq = pq_tq & 0b1111;

            tracing::debug!("Loading DQT entry with Pq={pq}, Tq={tq}");

            match pq {
                0 => {
                    let mut qk = [0; 64];
                    value
                        .read_exact(&mut qk)
                        .map_err(|_| Error::UnexpectedEof)?;
                    dqts.push(Self::Dqt8(Dqt_ { tq, qk }))
                }
                1 => {
                    let mut qk_raw = [0; 64 * 2];
                    value
                        .read_exact(&mut qk_raw)
                        .map_err(|_| Error::UnexpectedEof)?;

                    let mut qk = [0; 64];
                    for (n, i) in qk_raw.chunks_exact(2).enumerate() {
                        let entry = u16::from_be_bytes(i.try_into().unwrap());
                        qk[n] = entry;
                    }

                    dqts.push(Self::Dqt16(Dqt_ { tq, qk }))
                }
                unkown_pq => return Err(Error::UnknownPq(unkown_pq)),
            }
        }

        Ok(dqts)
    }
}

/// Frame
#[derive(Debug)]
pub struct Sof {
    /// Sample precision
    pub p: u8,
    /// Number of lines
    pub y: u16,
    /// Number of samples per line
    pub x: u16,
    /// Component specification parameters
    pub parameters: Vec<ComponentSpecificationParameters>,
}

impl Sof {
    pub fn from_data(mut data: &[u8]) -> Result<Self, Error> {
        let p = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let y = data.read_be_u16().map_err(|_| Error::UnexpectedEof)?;
        let x = data.read_be_u16().map_err(|_| Error::UnexpectedEof)?;
        let nf = data.read_u8().map_err(|_| Error::UnexpectedEof)?;

        let mut parameters = Vec::with_capacity(nf as usize);
        let buf = &mut [0; 3];
        for _ in 0..nf {
            data.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;
            parameters.push(ComponentSpecificationParameters::from_data(buf)?);
        }

        Ok(Self {
            p,
            y,
            x,
            parameters,
        })
    }
}

/// Component specification parameters
#[derive(Debug, Clone, Copy)]
pub struct ComponentSpecificationParameters {
    /// Component identifier
    pub c: u8,
    /// Horizontal sampling factor
    pub h: u8,
    /// Vertical sampling factor
    pub v: u8,
    /// Quantization table destination selector
    pub tq: u8,
}

impl ComponentSpecificationParameters {
    pub fn from_data(mut data: &[u8]) -> Result<Self, Error> {
        let c = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let h_v = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let tq = data.read_u8().map_err(|_| Error::UnexpectedEof)?;

        let h = h_v >> 4;
        let v = h_v & 0b1111;

        Ok(Self { c, h, v, tq })
    }
}

pub trait ReadExt: Read {
    fn read_u8(&mut self) -> std::io::Result<u8> {
        let buf = &mut [0; 1];
        self.read_exact(buf)?;
        Ok(buf[0])
    }

    fn read_be_u16(&mut self) -> std::io::Result<u16> {
        let buf = &mut [0; 2];
        self.read_exact(buf)?;
        Ok(u16::from_be_bytes(*buf))
    }
}

impl<T: Read> ReadExt for T {}

#[derive(Debug)]
pub struct Sos {
    pub components_specifications: Vec<ComponentSpecification>,
    pub ss: u8,
    pub se: u8,
    pub ah: u8,
    pub al: u8,
}

impl Sos {
    pub fn from_data(mut data: &[u8]) -> Result<Self, Error> {
        let ns = data.read_u8().map_err(|_| Error::UnexpectedEof)?;

        let mut components_specifications = Vec::with_capacity(ns as usize);
        let buf = &mut [0; 2];
        for _ in 0..ns {
            data.read_exact(buf).map_err(|_| Error::UnexpectedEof)?;
            components_specifications.push(ComponentSpecification::from_data(buf)?);
        }

        let ss = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let se = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let ah_al = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let ah = ah_al >> 4;
        let al = ah_al & 0b1111;

        Ok(Self {
            components_specifications,
            ss,
            se,
            ah,
            al,
        })
    }
}

#[derive(Debug)]
pub struct ComponentSpecification {
    /// Scan component selector
    ///
    /// References a [`c` value in
    /// `ComponentSpecificationParameters`](ComponentSpecificationParameters#
    /// structfield.c).
    pub cs: u8,
    /// DC entropy coding table
    pub td: u8,
    /// AC entropy coding table
    pub ta: u8,
}

impl ComponentSpecification {
    pub fn from_data(mut data: &[u8]) -> Result<ComponentSpecification, Error> {
        let cs = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let td_ta = data.read_u8().map_err(|_| Error::UnexpectedEof)?;
        let td = td_ta >> 4;
        let ta = td_ta & 0b1111;

        Ok(Self { cs, td, ta })
    }
}
