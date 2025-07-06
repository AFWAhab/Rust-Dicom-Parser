Relevant dicom tags for now 

0028,0010 Rows
0028,0011 Columns
7FE0,0010 Pixel Data

0010,0010 Patient Name
0008,0020 Study Date


TODO -> Add more support as we go along


# Adam notes
- The medical information encoded by a DICOM file is called a data set and takes the form of a key-value associative array. Each value can itself be a list of data sets (called a sequence), leading to a hierarchical data structure that is much like a XML or JSON file.
- Each key is called a dicom tag
- Dicom tags are nicknamed CamelCase (PatientName, StudyDescription)


Attribute Tag
A unique identifier for an Attribute of an Information Object composed of an ordered pair of numbers (a Group Number followed by an Element number).

Attribute Value
A Value of the Data Element corresponding to the Attribute of an Information Object.

Information Entity
That portion of information defined by a Composite IOD that is related to one specific class of Real-World Object. There is a one-to-one correspondence between Information Entities and entities in the DICOM Application Model.

Information Object Definition (IOD)
A data abstraction of a class of similar Real-World Objects that defines the nature and Attributes relevant to the class of Real-World Objects represented.

Big Endian A form of byte ordering where multiple byte binary values are encoded with the most significant
byte encoded first, and the remaining bytes encoded in decreasing order of significance.

Data Element A unit of information as defined by a single entry in the data dictionary. An encoded Information
Object Definition (IOD) Attribute that is composed of, at a minimum, three fields: a Data Element
Tag, a Value Length, and a Value Field. For some specific Transfer Syntaxes, a Data Element 
also contains a VR Field where the Value Representation of that Data Element is specified
explicitly.

Data Element Tag A unique identifier for a Data Element composed of an ordered pair of numbers (a Group Number
followed by an Element Number).

Data Element Tag Field The field within a Data Element structure that contains the Data Element Tag.


Pixel Cell The container for a single Pixel Sample Value that may include unused bits. The size of a Pixel
Cell shall be specified by the Bits Allocated (0028, 0100) Data Element.


Pixel Data Graphical data (e.g., images) of variable pixel-depth encoded in the Pixel Data, Float Pixel Data
or Double Float Pixel Data Element.



# FORSKEL PÅ ENDIAN
When storing multi-byte values (like a 16-bit u16 or a 32-bit u32) in binary file, there are two mains to order the bytes

Hex        Little Endian (LE)       Big Endian (BE)
0x1234     34 12                    12 34

Little endian stores the leat significant byt efirst (Intel/AMD CPUs us this) 
Big Endian stores the most significant byte first (Used more in networking or legacy systems)

In dicom the transfer syntax UID (tag (0002, 00010)) tells us whether the data that follows is little endian or big endian
Before that tag we assume little endian 


# Value Representation (Explicit vs Implicit)
VR = type of value

Dicom data element has this tructure
(Tag Group, Tag Element) - VR - Length - Value

VR tells us what kind of data this is
- PN (Person Name)
- DA (Date)
- US (Unsigned Short (16-bit))
- OB (Other byte (pixel blob))
- SQ Sequence (List of items) 

## Explicit VR
The file contains VR letters in the byte stream
(0010,0010) PN 08 "DOE^JOHN"

In bytes this would be something like

10 00 10 00    50 4E   08 00        44 4F 45 5E 4A 4F 48 4E
 TAG            VR      LEN                VALUE BYTES

## Implicit VR
Does not contain VR letters, so we have to look up the VR in the dicom dictionary baesd on the tag


10 00 10 00    08 00 00 00      44 4F 45 5E 4A 4F 48 4E
   TAG            LENGTH              Value Bytes


## Transfer Syntax
Transfer syntax (tag 0002,0010) tells us both:
UID	Name	Endianness	VR Type
"1.2.840.10008.1.2"	Implicit VR Little Endian	Little	Implicit
"1.2.840.10008.1.2.1"	Explicit VR Little Endian	Little	Explicit
"1.2.840.10008.1.2.2"	Explicit VR Big Endian	Big	Explicit

The parser must read the 0002,0010 tag (Transfer Syntax UID) in the header

Decide
- Do we read this as little or big endian
- Do i expect to see VR letteers or not
Use that info to parse the rest of the dataset properly
- 