use lopdf::Document;
use lopdf::Object;
use lopdf::Dictionary;

/// Embeds XML data into a PDF as a file attachment (ZUGFeRD/Factur-X compliant)
pub fn embed_xml_in_pdf(
    pdf_bytes: Vec<u8>,
    xml_data: &str,
    filename: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut document = Document::load_from(std::io::Cursor::new(pdf_bytes))?;

    // Create the embedded file stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Type", "EmbeddedFile");
    stream_dict.set("Length", xml_data.len() as i64);
    stream_dict.set("Subtype", "application/xml");

    let file_stream = Object::Stream(lopdf::Stream {
        content: xml_data.as_bytes().to_vec(),
        dict: stream_dict,
        allows_compression: true,
        start_position: None,
    });

    let file_stream_id = document.add_object(file_stream);

    let mut file_spec_dict = Dictionary::new();
    file_spec_dict.set("Type", "Filespec");
    file_spec_dict.set("F", filename);
    file_spec_dict.set("UF", filename);
    file_spec_dict.set("Desc", "Factur-X Invoice");

    let mut ef_dict = Dictionary::new();
    ef_dict.set("F", Object::Reference(file_stream_id));
    file_spec_dict.set("EF", Object::Dictionary(ef_dict));

    let file_spec_id = document.add_object(Object::Dictionary(file_spec_dict));

    let embedded_files_names = vec![
        Object::String(filename.as_bytes().to_vec(), Default::default()),
        Object::Reference(file_spec_id),
    ];

    let mut embedded_files_dict = Dictionary::new();
    embedded_files_dict.set("Names", embedded_files_names);

    let mut names_dict = Dictionary::new();
    names_dict.set("EmbeddedFiles", Object::Dictionary(embedded_files_dict));

    let names_id = document.add_object(Object::Dictionary(names_dict));

    let metadata_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:pdfaid="http://www.aiim.org/pdfa/ns/id/" xmlns:fx="urn:factur-x:pdfa" xmlns:pdfaExtension="http://www.aiim.org/pdfa/ns/extension/">
  <rdf:RDF>
    <rdf:Description rdf:about="">
      <pdfaid:part>3</pdfaid:part>
      <pdfaid:conformance>B</pdfaid:conformance>
    </rdf:Description>
    <rdf:Description rdf:about="">
      <pdfaExtension:schemas>
        <rdf:Bag>
          <rdf:li rdf:parseType="Resource">
            <pdfaExtension:schema>urn:factur-x:pdfa</pdfaExtension:schema>
            <pdfaExtension:prefix>fx</pdfaExtension:prefix>
            <pdfaExtension:namespaceURI>urn:factur-x:pdfa</pdfaExtension:namespaceURI>
            <pdfaExtension:properties>
              <rdf:Seq>
                <rdf:li rdf:parseType="Resource">
                  <pdfaExtension:name>ConformanceLevel</pdfaExtension:name>
                  <pdfaExtension:valueType>Text</pdfaExtension:valueType>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaExtension:name>DocumentType</pdfaExtension:name>
                  <pdfaExtension:valueType>Text</pdfaExtension:valueType>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaExtension:name>DocumentFileName</pdfaExtension:name>
                  <pdfaExtension:valueType>Text</pdfaExtension:valueType>
                </rdf:li>
                <rdf:li rdf:parseType="Resource">
                  <pdfaExtension:name>Version</pdfaExtension:name>
                  <pdfaExtension:valueType>Text</pdfaExtension:valueType>
                </rdf:li>
              </rdf:Seq>
            </pdfaExtension:properties>
          </rdf:li>
        </rdf:Bag>
      </pdfaExtension:schemas>
    </rdf:Description>
    <rdf:Description rdf:about="" xmlns:fx="urn:factur-x:pdfa">
      <fx:ConformanceLevel>Basic</fx:ConformanceLevel>
      <fx:DocumentType>INVOICE</fx:DocumentType>
      <fx:DocumentFileName>factur-x.xml</fx:DocumentFileName>
      <fx:Version>3.0</fx:Version>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>"#;

    let mut metadata_stream_dict = Dictionary::new();
    metadata_stream_dict.set("Type", "Metadata");
    metadata_stream_dict.set("Subtype", "XML");

    let metadata_stream = Object::Stream(lopdf::Stream {
        content: metadata_xml.as_bytes().to_vec(),
        dict: metadata_stream_dict,
        allows_compression: false,
        start_position: None,
    });

    let metadata_id = document.add_object(metadata_stream);

    for (_obj_id, obj) in document.objects.iter_mut() {
        if let Object::Dictionary(dict) = obj {
            // Check if this is the root catalog (has Type: Catalog)
            if let Ok(type_obj) = dict.get(b"Type") {
                if let Object::Name(name) = type_obj {
                    if name == b"Catalog" {
                        dict.set("Names", Object::Reference(names_id));
                        dict.set("Metadata", Object::Reference(metadata_id));

                        // Add AFRelationship for ZUGFeRD 2.0 compliance
                        let mut af_relationship = Dictionary::new();
                        af_relationship.set("Type", "AFRelationship");
                        af_relationship.set("Subtype", "Data");
                        af_relationship.set("AF", Object::Array(vec![Object::Reference(file_spec_id)]));

                        let af_array = vec![Object::Dictionary(af_relationship)];
                        dict.set("AF", Object::Array(af_array));

                        break;
                    }
                }
            }
        }
    }

    let mut output = Vec::new();
    document.save_to(&mut output)?;

    Ok(output)
}
