use crate::utils::{IncompleteInt, InputStream};
use crate::{sbs1, sbs2};
use core::num::NonZeroU64;

/*
mod encode {
    use super::*;

    fn fits_next(value: u64) -> bool {
        unimplemented!()
    }

    fn write(value: u64) {
        unimplemented!()
    }
}
*/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SbsMarker {
    Mode1,
    Mode2,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum RhoRegion {
    Eq0,
    Leq1div3,
    Gthan1d3Lthan3d4,
    Geq3Div4,
    Eq1,
}

impl RhoRegion {
    pub fn next(self, value: NonZeroU64) -> Self {
        match (self, value.get()) {
            (Self::Eq0, 1) => Self::Eq1,
            (Self::Leq1div3, 1) => Self::Geq3Div4,
            (_, 1) => Self::Gthan1d3Lthan3d4,
            (Self::Eq1, 2) => Self::Leq1div3,
            (_, 2) => Self::Gthan1d3Lthan3d4,
            _ => Self::Leq1div3,
        }
    }

    pub fn which(self) -> SbsMarker {
        match self {
            Self::Eq0 | Self::Leq1div3 | Self::Gthan1d3Lthan3d4 => SbsMarker::Mode1,
            Self::Geq3Div4 | Self::Eq1 => SbsMarker::Mode2,
        }
    }
}

pub struct Coder(RhoRegion);

impl Coder {
    pub fn skip(&mut self) -> bool {
        unimplemented!()
    }

    pub fn read<'a>(
        &mut self,
        stream: (&'a [u8], usize),
    ) -> Result<(InputStream<'a>, NonZeroU64), IncompleteInt<NonZeroU64>> {
        let result = match self.0.which() {
            SbsMarker::Mode1 => sbs1::decode::read(stream),
            SbsMarker::Mode2 => sbs2::decode::read(stream),
        };
        if let Ok((_, value)) = result {
            self.0 = self.0.next(value);
        }

        result
    }
}
