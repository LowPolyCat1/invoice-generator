use lopdf::Document;
use lopdf::Object;
use lopdf::Dictionary;
use std::process::Command;

pub fn convert_to_pdfa3(pdf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if add_pdfa3_metadata(pdf_path).is_ok() {
        return Ok(());
    }

    Err("Could not convert PDF to PDF/A-3. Install Ghostscript or qpdf for full ZUGFeRD compliance.".into())
}

fn add_pdfa3_metadata(pdf_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pdf_bytes = std::fs::read(pdf_path)?;
    let mut document = Document::load_from(std::io::Cursor::new(pdf_bytes))?;

    let xmp_metadata = create_pdfa3_xmp_metadata();

    let mut metadata_stream_dict = Dictionary::new();
    metadata_stream_dict.set("Type", "Metadata");
    metadata_stream_dict.set("Subtype", "XML");
    metadata_stream_dict.set("Length", xmp_metadata.len() as i64);

    let metadata_stream = Object::Stream(lopdf::Stream {
        content: xmp_metadata.as_bytes().to_vec(),
        dict: metadata_stream_dict,
        allows_compression: false,
        start_position: None,
    });

    let metadata_id = document.add_object(metadata_stream);

    for (_obj_id, obj) in document.objects.iter_mut() {
        if let Object::Dictionary(dict) = obj {
            if let Ok(type_obj) = dict.get(b"Type") {
                if let Object::Name(name) = type_obj {
                    if name == b"Catalog" {
                        // Set metadata
                        dict.set("Metadata", Object::Reference(metadata_id));

                        // Ensure OutputIntents for PDF/A-3 compliance
                        if dict.get(b"OutputIntents").is_err() {
                            let mut output_intent = Dictionary::new();
                            output_intent.set("Type", "OutputIntent");
                            output_intent.set("S", "GTS_PDFA1");
                            output_intent.set("OutputConditionIdentifier", "sRGB IEC61966-2.1");
                            output_intent.set("RegistryName", "http://www.color.org");
                            output_intent.set("Info", "sRGB color space");

                            dict.set("OutputIntents", Object::Array(vec![Object::Dictionary(output_intent)]));
                        }

                        break;
                    }
                }
            }
        }
    }

    let mut output = Vec::new();
    document.save_to(&mut output)?;
    std::fs::write(pdf_path, output)?;

    Ok(())
}

fn create_pdfa3_xmp_metadata() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/">
  <rdf:RDF>
    <rdf:Description rdf:about="">
      <pdfaid:part>3</pdfaid:part>
      <pdfaid:conformance>b</pdfaid:conformance>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#.to_string()
}
