Rust Dicom Parser, personal project


Current TODOS:
- Refactor parser into functions
- Decode JPEG image fragments
- Pretty print tags (Patient Name, Study Date, Modality, Dimensions, Bits Allocated)
- Implicit VR Support (After parsing Transfer Syntax UID, if it is 1.2.840.10008.1.2 -> skip VR field and lookup in dictionary)
- Unit testing (Explicit VR Big Endian, RLE Compressed Pixel Data, Multi-frame images, No DICM preamble
- Refactor (Minor cleanups)
