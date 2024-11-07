impl crate::Jpeg {
    pub fn encoder<W: jpeg_encoder::JfifWrite>(
        &self,
        w: W,
    ) -> Result<jpeg_encoder::Encoder<W>, EncoderError> {
        let mut encoder = jpeg_encoder::Encoder::new(w, 100);

        encoder.set_sampling_factor(self.sampling_factor()?);

        //encoder.set_quantization_tables(luma, chroma);

        //encoder.set_progressive(progressive);

        //encoder.set_restart_interval(interval);

        encoder.set_optimized_huffman_tables(true);

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
