use std::io;
use std::fs::File;
use std::io::Read;
use std::io::Error;
use std::io::ErrorKind;

fn main() -> io::Result<()> {
    let mut file = File::open("dicom_images/image-000001.dcm")?;
    let mut buffer = Vec::new();

    // Makes the buffer to be a Vec<u8>
    // Though &T is a shared read only reference,
    // &mut T is an exclusive writeable reference, which means that &mut buffer can modify the buffer even though it doesnt own it
    file.read_to_end(&mut buffer)?;

    // Handling pre-abmle, 127 bytes are 0's until we see "68 73 67 77" which is the "DICM" tag.
    let mut seen_dicm_tag: bool = false;

    if &buffer[128..132] == b"DICM" {
        seen_dicm_tag = true;
        println!("DICM tag found!")
    } else {
        println!("DICM is not found");
        return Err(Error::new(ErrorKind::InvalidData, "Missing Dicm Tag"))
    }


    let mut cursor: usize = 132; // Skipping the preamble + DICM
    // TODO -> Fix this later, because we know that our files are DICM files, but if they are not, we want to error handle that

    // Iterating through the file
    // Checking 8 bytes = minimum tag + VR & VL
    while cursor + 8 < buffer.len() {
        // Read Tag (Group, Element)
        // Reading tag fields (4 byte tag, e.g. (0002,0010))
        let group = u16::from_le_bytes([buffer[cursor], buffer[cursor+1]]);
        let element = u16::from_le_bytes([buffer[cursor+2], buffer[cursor+3]]);
        cursor +=4;

        // Reading the Value Representation
        if cursor + 2 > buffer.len() {
            panic!("Unexpected end of buffer while reading VR")
        }

        let vr = &buffer[cursor..cursor + 2];
        let vr_str = std::str::from_utf8(vr).unwrap_or("???");
        cursor += 2;

        // Decide Length Field Size
        let extended_vrs = ["OB", "OW", "OF", "SQ", "UT", "UN"];
        let length: usize;

        if extended_vrs.contains(&std::str::from_utf8(vr).unwrap()) {

            // Skip Reserved Bytes
            cursor += 2;

            length = u32::from_le_bytes([
                buffer[cursor],
                buffer[cursor + 1],
                buffer[cursor + 2],
                buffer[cursor + 3],
            ]) as usize;
            cursor += 4;
        } else {
            if cursor + 2 > buffer.len() {
                panic!("Unexpected end of buffer while reading VL");
            }

            length = u16::from_le_bytes([buffer[cursor], buffer[cursor+1]]) as usize;
            cursor += 2;
        }

        println!("Tag: ({:04X},{:04X}), VR: {}, Length: {} at offset {}",
                group, element, vr_str, length, cursor
        );

        // Apparently some of the dicom elements, can ahve undefined length, so if the lengh is 0xFFFFFFFF, then we have to scan
        if length == 0xFFFFFFFF {
            println!("Undefined length, beginning item scan");

            loop {
                // Read item tag
                let item_group = u16::from_le_bytes([buffer[cursor], buffer[cursor + 1]]);
                let item_element = u16::from_le_bytes([buffer[cursor + 2], buffer[cursor + 3]]);
                cursor += 4;

                let item_length = u32::from_le_bytes([
                    buffer[cursor],
                    buffer[cursor + 1],
                    buffer[cursor + 2],
                    buffer[cursor + 3],
                ]) as usize;
                cursor += 4;

                println!("Item tag: ({:04X},{:04X}), Length = {}", item_group, item_element, item_length);

                /*
                There are three special SQ related Data Elements that are not ruled by the VR encoding rules conveyed by the Transfer Syntax. They
                shall be encoded as Implicit VR. These special Data Elements are Item (FFFE,E000), Item Delimitation Item (FFFE,E00D), and Sequence
                Delimitation Item (FFFE,E0DD). However, the Data Set within the Value Field of the Data Element Item (FFFE,E000) shall
                be encoded according to the rules conveyed by the Transfer Syntax
                */
                /*
                Each Item of a Data Element of Value Representation SQ shall be encoded as a DICOM Standard Data Element with a specific Data
                 Element Tag of Value (FFFE,E000). The Item Tag is followed by a 4 byte Value (Item) Length field encoded in one of the following
                 two ways:
                 a. Explicit Length: The number of bytes (even) contained in the Sequence Item Value (following but not including the Value (Item)
                 Length Field) is encoded as a 32-bit unsigned integer value (see Section 7.1). This length shall include the total length of all Data
                 Elements conveyed by this Item. This Value (Item) Length Field shall be equal to 00000000H if the Item contains no Data Set.
                 b. Undefined Length: The Value (Item) Length Field shall contain the value FFFFFFFFH to indicate an Undefined Length. It shall
                 be used in conjunction with an Item Delimitation Data Element. This Item Delimitation Data Element has a Data Element Tag of
                 (FFFE,E00D) and shall follow the Data Elements encapsulated in the Item. No Value shall be present in the Item Delimitation
                 Data Element and its Value (Item) Length shall be 00000000H. An Item containing no Data Set is encoded by an Item Delimitation
                 Data Element only.
                */

                if item_group == 0xFFFE && item_element == 0xE0DD {
                    println!("Sequence delimitation item, done wiht undefined length value");
                    break;
                } else if item_group == 0xFFFE && item_element == 0xE000 {
                    // Item start - skip the data
                    if cursor + item_length > buffer.len() {
                        panic!("Item data overruns buffer: cursor={} + item_length={} > buffer.len()={}", cursor, item_length, buffer.len());
                    }
                    cursor += item_length;
                } else {
                    println!("Unknown FFFE tag: ({:04X}, {:04X}), skipping {} bytes", item_group, item_element, item_length);
                    cursor += item_length
                }
            }
            continue;
        }

        if length > 10_000_000 {
            println!("Too large length {}", length)
        }

        if cursor + length > buffer.len() {
            panic!("Trying to read past end of buffer: cursor={} + length={} > buffer.len()={}", cursor, length, buffer.len());
        }


        // Read Value
        let value_bytes = &buffer[cursor.. cursor + length];
        cursor += length;

        // Check for transfer syntax uid
        if group == 0x0002 && element == 0x00010 {
            let transfer_syntax_uid = std::str::from_utf8(value_bytes).unwrap();
            println!("Transfer Syntax UID: {}", transfer_syntax_uid)
        }
    }
    //println!("First 200 bytes: {:?}", &buffer[..]);
    Ok(())
}