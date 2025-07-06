use std::fs::File;
use std::io::{self, Error, ErrorKind, Read};

#[derive(Debug)]
struct DataElement<'a> {
    tag: (u16, u16),
    vr: String,
    length: usize,
    value: Option<&'a [u8]>,
    offset: usize,
}

fn main() -> io::Result<()> {
    let mut file = File::open("dicom_images/image-000001.dcm")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if &buffer[128..132] != b"DICM" {
        return Err(Error::new(ErrorKind::InvalidData, "Missing DICM tag"));
    }
    println!("DICM tag found!");

    parse_dicom(&buffer)?;
    Ok(())
}

fn parse_dicom(buffer: &[u8]) -> io::Result<()> {
    let mut cursor = 132; // Skipping the DICM Preamble

    while cursor + 8 < buffer.len() {
        let tag = read_tag(&buffer, &mut cursor)?;
        let vr = read_vr(&buffer, &mut cursor)?; // Reading the Value Representation, which tells us what kind of data this is
        let length = read_length(&buffer, &mut cursor, &vr)?;
        let offset = cursor;

        let value = if length == 0xFFFFFFFF {
            handle_undefined_length(&buffer, &mut cursor)?;
            None
        } else {
            if cursor + length > buffer.len() {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Value overruns buffer"));
            }
            let val = &buffer[cursor..cursor + length];
            cursor += length;
            Some(val)
        };

        let element = DataElement {
            tag,
            vr,
            length,
            value,
            offset,
        };

        print_element(&element);
    }

    Ok(())
}

fn read_tag(buffer: &[u8], cursor: &mut usize) -> io::Result<(u16, u16)> {
    if *cursor + 4 > buffer.len() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "Tag out of bounds"));
    }
    let group = u16::from_le_bytes([buffer[*cursor], buffer[*cursor + 1]]);
    let element = u16::from_le_bytes([buffer[*cursor + 2], buffer[*cursor + 3]]);
    *cursor += 4;
    Ok((group, element))
}

fn read_vr(buffer: &[u8], cursor: &mut usize) -> io::Result<String> {
    let vr_bytes = &buffer[*cursor..*cursor + 2];
    *cursor += 2;
    Ok(std::str::from_utf8(vr_bytes).unwrap_or("??").to_string())
}

fn read_length(buffer: &[u8], cursor: &mut usize, vr: &str) -> io::Result<usize> {
    let extended_vrs = ["OB", "OW", "OF", "SQ", "UT", "UN"];
    if extended_vrs.contains(&vr) {
        *cursor += 2; // reserved bytes
        Ok(u32::from_le_bytes([
            buffer[*cursor],
            buffer[*cursor + 1],
            buffer[*cursor + 2],
            buffer[*cursor + 3],
        ]) as usize)
    } else {
        Ok(u16::from_le_bytes([buffer[*cursor], buffer[*cursor + 1]]) as usize)
    }.map(|len| {
        *cursor += if extended_vrs.contains(&vr) { 4 } else { 2 };
        len
    })
}

fn handle_undefined_length(buffer: &[u8], cursor: &mut usize) -> io::Result<()> {
    println!("📦 Encountered undefined length — beginning item scan…");

    loop {
        let group = u16::from_le_bytes([buffer[*cursor], buffer[*cursor + 1]]);
        let element = u16::from_le_bytes([buffer[*cursor + 2], buffer[*cursor + 3]]);
        *cursor += 4;

        let length = u32::from_le_bytes([
            buffer[*cursor],
            buffer[*cursor + 1],
            buffer[*cursor + 2],
            buffer[*cursor + 3],
        ]) as usize;
        *cursor += 4;

        println!("Item tag: ({:04X},{:04X}), Length = {}", group, element, length);

        if group == 0xFFFE && element == 0xE0DD {
            println!("Sequence Delimitation Item is now finished.");
            break;
        }

        *cursor += length;
    }

    Ok(())
}

fn print_element(el: &DataElement) {
    println!(
        "Tag: ({:04X},{:04X}), VR: {}, Length: {}, Offset: {}, Value: {}",
        el.tag.0,
        el.tag.1,
        el.vr,
        el.length,
        el.offset,
        match el.value {
            Some(val) if val.len() <= 16 => format!("{:?}", val),
            Some(_) => "<...>".to_string(),
            None => "<undefined length>".to_string(),
        }
    );
}