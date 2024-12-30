use crate::Error;
use gufo_common::math::*;

impl crate::Jpeg {
    pub fn encoder<W: jpeg_encoder::JfifWrite>(
        &self,
        w: W,
    ) -> Result<jpeg_encoder::Encoder<W>, EncoderError> {
        let mut encoder = jpeg_encoder::Encoder::new(w, 50);

        encoder.set_sampling_factor(self.sampling_factor()?);

        let (luma, chroma) = self.quantization_tables()?;

        encoder.set_quantization_tables(luma, chroma);

        let progressive = self.is_progressive()?;
        encoder.set_progressive(progressive);
        if progressive {
            encoder.set_progressive_scans(self.n_sos().u8().map_err(|x| Error::from(x))?);
        }

        Ok(encoder)
    }

    pub fn sampling_factor(&self) -> Result<jpeg_encoder::SamplingFactor, EncoderError> {
        match self.color_model()?.color_type()? {
            jpeg_encoder::JpegColorType::Cmyk => {
                let parameters = self.components_specification_parameters(3)?;
                jpeg_encoder::SamplingFactor::from_factors(parameters.h, parameters.v)
            }
            jpeg_encoder::JpegColorType::Luma => Some(jpeg_encoder::SamplingFactor::F_1_1),
            jpeg_encoder::JpegColorType::Ycbcr => {
                let parameters = self.components_specification_parameters(0)?;
                jpeg_encoder::SamplingFactor::from_factors(parameters.h, parameters.v)
            }
            jpeg_encoder::JpegColorType::Ycck => {
                let parameters_0 = self.components_specification_parameters(0)?;
                let parameters_3 = self.components_specification_parameters(3)?;

                if parameters_0.h != parameters_3.h || parameters_0.v != parameters_3.v {
                    return Err(EncoderError::UnsupportedSamplingFactor);
                }

                jpeg_encoder::SamplingFactor::from_factors(parameters_0.h, parameters_0.v)
            }
        }
        .ok_or(EncoderError::UnsupportedSamplingFactor)
    }

    pub fn quantization_tables(
        &self,
    ) -> Result<
        (
            jpeg_encoder::QuantizationTableType,
            jpeg_encoder::QuantizationTableType,
        ),
        EncoderError,
    > {
        let dqts = self.dqts()?;

        let luma_parameters = self.components_specification_parameters(0)?;
        let luma_table = dqts.get(&luma_parameters.tq).ok_or(Error::MissingDqt)?;
        let luma = jpeg_encoder::QuantizationTableType::Custom(Box::new(luma_table.qk()));

        let chroma_parameters = self.components_specification_parameters(1)?;
        let chroma_table = dqts.get(&chroma_parameters.tq).ok_or(Error::MissingDqt)?;
        let chroma = jpeg_encoder::QuantizationTableType::Custom(Box::new(chroma_table.qk()));

        Ok((luma, chroma))
    }
}

impl crate::ColorModel {
    pub fn color_type(&self) -> Result<jpeg_encoder::JpegColorType, EncoderError> {
        match self {
            Self::Cmyk => Ok(jpeg_encoder::JpegColorType::Cmyk),
            Self::Grayscale => Ok(jpeg_encoder::JpegColorType::Luma),
            Self::Rgb => Err(EncoderError::UnsupportedColorModel),
            Self::YCbCr => Ok(jpeg_encoder::JpegColorType::Ycbcr),
            Self::Ycck => Ok(jpeg_encoder::JpegColorType::Ycck),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncoderError {
    #[error("{0}")]
    Gufo(#[from] crate::Error),
    #[error("Color model not supported by encoder")]
    UnsupportedColorModel,
    #[error("Sampling factor not supported by encoder")]
    UnsupportedSamplingFactor,
}
