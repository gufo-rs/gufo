use std::{borrow::Cow, io::Cursor};

use gufo_common::{error::ErrorWithData, image::ImageMetadata};
use xml::{
    EmitterConfig, ParserConfig,
    name::OwnedName,
    reader::{self, XmlEvent},
    writer,
};

#[derive(Debug)]
pub struct Svg {
    data: Vec<u8>,
}

impl ImageMetadata for Svg {
    fn xmp(&self) -> Vec<Vec<u8>> {
        let Ok(Some(xmp)) = self.parse_xmp() else {
            return Vec::new();
        };

        vec![xmp]
    }
}

impl Svg {
    pub fn new(data: Vec<u8>) -> Result<Self, ErrorWithData<Error>> {
        Ok(Self { data })
    }

    pub fn is_filetype(data: &[u8]) -> bool {
        let Some(svg_tag) = memchr::memmem::find(data, b"<svg") else {
            return false;
        };
        let Some(svg_namespace) = memchr::memmem::find(data, b"http://www.w3.org/2000/svg") else {
            return false;
        };

        svg_tag < svg_namespace
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.data
    }

    /// Make an XMP document out of the <metadata> tag
    fn parse_xmp(&self) -> Result<Option<Vec<u8>>, Error> {
        let parser = ParserConfig::default()
            .ignore_root_level_whitespace(false)
            .create_reader(Cursor::new(&self.data));

        let mut output = Vec::new();
        let mut writer = EmitterConfig::default()
            .write_document_declaration(false)
            .pad_self_closing(false)
            .create_writer(&mut output);

        writer.write(writer::XmlEvent::ProcessingInstruction {
            name: "xpacket",
            data: Some(r#"begin="" id="W5M0MpCehiHzreSzNTczkc9d""#),
        })?;

        let mut state = State::Initial;
        let mut found_rdf = false;

        for event in parser {
            match event? {
                XmlEvent::EndElement { name }
                    if name.namespace.as_deref() == Some(NS_RDF) && name.local_name == "RDF" =>
                {
                    // Early match to stop copying events when we are no longerin RDF
                    state = State::InMetadata;
                    writer.write(writer::XmlEvent::EndElement {
                        name: Some(name.borrow()),
                    })?;
                }
                event if matches!(state, State::InRdf) => {
                    // Just copy the events inside of RDF elements
                    writer.write(event.as_writer_event().unwrap())?;
                }
                XmlEvent::StartElement {
                    name,
                    attributes: _,
                    mut namespace,
                } => match &state {
                    State::Initial => {
                        if name.namespace.as_deref() == Some(NS_SVG) && name.local_name == "svg" {
                            state = State::InSvg;
                            // TODO: Support shifting to a different prefix if already used
                            namespace.0.insert(String::from("x"), NS_XMP.to_string());
                            // Reinterpret the <xml> tag as <xmpmeta> tag, this way we can copy the namespace defintions
                            let name = OwnedName {
                                local_name: String::from("xmpmeta"),
                                namespace: None,
                                prefix: Some(String::from("x")),
                            };
                            writer.write(writer::XmlEvent::StartElement {
                                name: name.borrow(),
                                attributes: Default::default(),
                                namespace: Cow::Owned(namespace),
                            })?;
                        }
                    }
                    State::InSvg => {
                        if name.namespace.as_deref() == Some(NS_SVG)
                            && name.local_name == "metadata"
                        {
                            // TODO: In theory, this element could have namespace definitions that we need to keep
                            state = State::InMetadata;
                        }
                    }
                    State::InMetadata => {
                        if name.namespace.as_deref() == Some(NS_RDF) && name.local_name == "RDF" {
                            state = State::InRdf;
                            found_rdf = true;
                            writer.write(writer::XmlEvent::StartElement {
                                name: name.borrow(),
                                // TODO: do we want the attributes?
                                attributes: Default::default(),
                                namespace: Cow::Owned(namespace),
                            })?;
                        }
                    }
                    _ => {}
                },
                XmlEvent::EndElement { name } => {
                    if name.namespace.as_deref() == Some(NS_SVG) && name.local_name == "svg" {
                        state = State::Initial;
                        let name = OwnedName {
                            local_name: String::from("xmpmeta"),
                            namespace: None,
                            prefix: Some(String::from("x")),
                        };
                        writer.write(writer::XmlEvent::EndElement {
                            name: Some(name.borrow()),
                        })?;
                    } else if name.namespace.as_deref() == Some(NS_SVG)
                        && name.local_name == "metadata"
                    {
                        state = State::InSvg;
                    }
                }
                _ => {}
            }
        }

        writer.write(writer::XmlEvent::ProcessingInstruction {
            name: "xpacket",
            data: Some(r#"end="""#),
        })?;

        // Only claim we found XMP data if there was an RDF tag
        Ok(found_rdf.then_some(output))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XmlWriterError: {0}")]
    WriterError(#[from] writer::Error),
    #[error("XmlReaderError: {0}")]
    ReaderError(#[from] reader::Error),
}

#[derive(Debug)]
pub enum State {
    Initial,
    InSvg,
    InMetadata,
    InRdf,
}

const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const NS_SVG: &str = "http://www.w3.org/2000/svg";
const NS_XMP: &str = "adobe:ns:meta/";
