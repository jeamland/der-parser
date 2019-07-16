/// Combination and flat_map! and take! as first combinator
#[macro_export]
macro_rules! flat_take (
    ($i:expr, $len:expr, $f:ident) => ({
        if $i.len() < $len { Err(::nom::Err::Incomplete(::nom::Needed::Size($len))) }
        else {
            let taken = &$i[0..$len];
            let rem = &$i[$len..];
            match $f(taken) {
                Ok((_,res)) => Ok((rem,res)),
                Err(e)      => Err(e)
            }
        }
    });
    ($i:expr, $len:expr, $submac:ident!( $($args:tt)*)) => ({
        if $i.len() < $len { Err(::nom::Err::Incomplete(::nom::Needed::Size($len))) }
        else {
            let taken = &$i[0..$len];
            let rem = &$i[$len..];
            match $submac!(taken, $($args)*) {
                Ok((_,res)) => Ok((rem,res)),
                Err(e)      => Err(e)
            }
        }
    });
);

/// Internal parser, do not use directly
#[doc(hidden)]
#[macro_export]
macro_rules! fold_der_defined_m(
    (__impl $i:expr, $acc:ident, $f:ident) => ( {
        match $f($i) {
            Ok((rem,res)) => { $acc.push(res); Ok((rem,$acc)) },
            Err(e)        => Err(e)
        }
    });
    (__impl $i:expr, $acc:ident, $submac:ident!( $($args:tt)* ) ) => ( {
        match $submac!($i, $($args)*) {
            Ok((rem,res)) => { $acc.push(res); Ok((rem,$acc)) },
            Err(e)        => Err(e)
        }
    });
    (__impl $i:expr, $acc:ident, $f:ident >> $($rest:tt)*) => (
        {
            match $f($i) {
                Ok((rem,res)) => {
                    $acc.push(res);
                    fold_der_defined_m!(__impl rem, $acc, $($rest)* )
                },
                Err(e)        => Err(e)
            }
        }
    );
    (__impl $i:expr, $acc:ident, $submac:ident!( $($args:tt)* ) >> $($rest:tt)*) => (
        {
            match $submac!($i, $($args)*) {
                Ok((rem,res)) => {
                    $acc.push(res);
                    fold_der_defined_m!(__impl rem, $acc, $($rest)* )
                },
                Err(e)        => Err(e)
            }
        }
    );

    ($i:expr, $($rest:tt)* ) => (
        {
            let mut v = Vec::new();
            fold_der_defined_m!(__impl $i, v, $($rest)*)
        }
    );
);

/// Internal parser, do not use directly
#[doc(hidden)]
#[macro_export]
macro_rules! parse_ber_defined_m(
    ($i:expr, $tag:expr, $($args:tt)*) => (
        {
            use $crate::ber::ber_read_element_header;
            do_parse!(
                $i,
                hdr:     ber_read_element_header >>
                         error_if!(hdr.class != 0b00, ErrorKind::Custom($crate::error::BER_CLASS_ERROR)) >>
                         error_if!(hdr.structured != 0b1, ErrorKind::Custom($crate::error::BER_STRUCT_ERROR)) >>
                         error_if!(hdr.tag != $tag, ErrorKind::Custom($crate::error::BER_TAG_ERROR)) >>
                content: flat_take!(hdr.len as usize, fold_der_defined_m!( $($args)* )) >>
                (hdr,content)
            )
        }
    );
);

/// Parse a sequence of DER elements (macro version)
///
/// Unlike [`parse_der_sequence`](fn.parse_der_sequence.html), this function allows to specify the
/// list of expected types in the DER sequence.
///
/// Similar to [`parse_der_sequence_defined`](macro.parse_der_sequence_defined.html), but not using `fold`.
/// This allow using macros.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn localparse_seq(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_sequence_defined_m!(i,
///         parse_ber_integer >>
///         call!(parse_ber_integer)
///     )
/// }
/// let empty = &b""[..];
/// let bytes = [ 0x30, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_seq(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(localparse_seq(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_sequence_defined_m(
    ($i:expr, $($args:tt)*) => ({
        map!(
            $i,
            parse_ber_defined_m!($crate::ber::BerTag::Sequence, $($args)*),
            |(hdr,o)| $crate::ber::BerObject::from_header_and_content(hdr,$crate::ber::BerObjectContent::Sequence(o))
        )
    });
);

/// Parse a set of DER elements (macro version)
///
/// Unlike [`parse_der_set`](fn.parse_der_set.html), this function allows to specify the
/// list of expected types in the DER set.
///
/// Similar to [`parse_der_set_defined`](macro.parse_der_set_defined.html), but not using `fold`.
/// This allow using macros.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn localparse_set(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_set_defined_m!(i,
///         parse_ber_integer >>
///         call!(parse_ber_integer)
///     )
/// }
/// let empty = &b""[..];
/// let bytes = [ 0x31, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_set(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(localparse_set(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_set_defined_m(
    ($i:expr, $($args:tt)*) => ({
        map!(
            $i,
            parse_ber_defined_m!($crate::ber::BerTag::Set, $($args)*),
            |(hdr,o)| $crate::ber::BerObject::from_header_and_content(hdr,$crate::ber::BerObjectContent::Set(o))
        )
    });
);

/// Internal parser, do not use directly
#[doc(hidden)]
#[macro_export]
macro_rules! fold_parsers(
    ($i:expr, $($args:tt)*) => (
        {
            let parsers = [ $($args)* ];
            parsers.iter().fold(
                (Ok(($i,vec![]))),
                |r, f| {
                    match r {
                        Ok((rem,mut v)) => {
                            map!(rem, f, |x| { v.push(x); v })
                        }
                        Err(e)          => Err(e)
                    }
                }
                )
        }
    );
);

/// Internal parser, do not use directly
#[doc(hidden)]
#[macro_export]
macro_rules! parse_der_defined(
    ($i:expr, $ty:expr, $($args:tt)*) => (
        {
            use $crate::ber::ber_read_element_header;
            use nom::ErrorKind;
            let res =
            do_parse!(
                $i,
                hdr:     ber_read_element_header >>
                         error_if!(hdr.class != 0b00, ErrorKind::Custom($crate::error::BER_CLASS_ERROR)) >>
                         error_if!(hdr.structured != 0b1, ErrorKind::Custom($crate::error::BER_STRUCT_ERROR)) >>
                         error_if!(hdr.tag != $ty, ErrorKind::Custom($crate::error::BER_TAG_ERROR)) >>
                content: take!(hdr.len) >>
                (hdr,content)
            );
            match res {
                Ok((_rem,o)) => {
                    match fold_parsers!(o.1, $($args)* ) {
                        Ok((rem,v)) => {
                            if rem.len() != 0 { Err(::nom::Err::Error(error_position!($i, ErrorKind::Custom($crate::error::BER_OBJ_TOOSHORT)))) }
                            else { Ok((_rem,(o.0,v))) }
                        },
                        Err(e)      => Err(e)
                    }
                },
                Err(e)       => Err(e)
            }
        }
    );
);

/// Parse a sequence of DER elements (folding version)
///
/// Unlike [`parse_der_sequence`](fn.parse_der_sequence.html), this function allows to specify the
/// list of expected types in the DER sequence.
///
/// Similar to [`parse_der_sequence_defined_m`](macro.parse_der_sequence_defined_m.html), but uses
/// `fold` internally.
/// Because of that, macros cannot be used as subparsers.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn localparse_seq(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_sequence_defined!(i,
///         parse_ber_integer,
///         parse_ber_integer
///     )
/// }
/// let empty = &b""[..];
/// let bytes = [ 0x30, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_seq(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(localparse_seq(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_sequence_defined(
    ($i:expr, $($args:tt)*) => (
        map!(
            $i,
            parse_der_defined!($crate::ber::BerTag::Sequence, $($args)*),
            |(hdr,o)| $crate::ber::BerObject::from_header_and_content(hdr,$crate::ber::BerObjectContent::Sequence(o))
        )
    );
);

/// Parse a set of DER elements (folding version)
///
/// Unlike [`parse_der_set`](fn.parse_der_set.html), this function allows to specify the
/// list of expected types in the DER sequence.
///
/// Similar to [`parse_der_set_defined_m`](macro.parse_der_set_defined_m.html), but uses
/// `fold` internally.
/// Because of that, macros cannot be used as subparsers.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn localparse_set(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_set_defined!(i,
///         parse_ber_integer,
///         parse_ber_integer
///     )
/// }
/// let empty = &b""[..];
/// let bytes = [ 0x31, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_set(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(localparse_set(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_set_defined(
    ($i:expr, $($args:tt)*) => (
        map!(
            $i,
            parse_der_defined!($crate::ber::BerTag::Set, $($args)*),
            |(hdr,o)| $crate::ber::BerObject::from_header_and_content(hdr,$crate::ber::BerObjectContent::Set(o))
        )
    );
);

/// Parse a sequence of identical DER elements
///
/// Given a subparser for a DER type, parse a sequence of identical objects.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn parser(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_sequence_of!(i, parse_ber_integer)
/// };
/// let empty = &b""[..];
/// let bytes = [ 0x30, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_seq(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(parser(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_sequence_of(
    ($i:expr, $f:ident) => ({
        use $crate::ber::ber_read_element_header;
        do_parse!(
            $i,
            hdr:     ber_read_element_header >>
                     error_if!(hdr.tag != $crate::ber::BerTag::Sequence, ErrorKind::Custom($crate::error::BER_TAG_ERROR)) >>
            content: flat_take!(hdr.len as usize,
                do_parse!(
                    r: many0!(complete!($f)) >>
                       empty!() >>
                    ( r )
                )
            ) >>
            ( $crate::ber::BerObject::from_header_and_content(hdr, $crate::ber::BerObjectContent::Sequence(content)) )
        )
    })
);

/// Parse a set of identical DER elements
///
/// Given a subparser for a DER type, parse a set of identical objects.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn parser(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_set_of!(i, parse_ber_integer)
/// };
/// let empty = &b""[..];
/// let bytes = [ 0x31, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let expected  = BerObject::from_set(vec![
///     BerObject::from_int_slice(b"\x01\x00\x01"),
///     BerObject::from_int_slice(b"\x01\x00\x00"),
/// ]);
/// assert_eq!(parser(&bytes), Ok((empty, expected)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_set_of(
    ($i:expr, $f:ident) => ({
        use $crate::ber::ber_read_element_header;
        do_parse!(
            $i,
            hdr:     ber_read_element_header >>
                     error_if!(hdr.tag != $crate::ber::BerTag::Set, ErrorKind::Custom($crate::error::BER_TAG_ERROR)) >>
            content: flat_take!(hdr.len as usize,
                do_parse!(
                    r: many0!(complete!($f)) >>
                       empty!() >>
                    ( r )
                )
            ) >>
            ( $crate::ber::BerObject::from_header_and_content(hdr, $crate::ber::BerObjectContent::Set(content)) )
        )
    })
);

/// Parse an optional DER element
///
/// Try to parse an optional DER element, and return it as a `ContextSpecific` item with tag 0.
/// If the parsing failed, the `ContextSpecific` object has value `None`.
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// let empty = &b""[..];
/// let bytes1 = [ 0x30, 0x0a,
///                0x0a, 0x03, 0x00, 0x00, 0x01,
///                0x02, 0x03, 0x01, 0x00, 0x01];
/// let bytes2 = [ 0x30, 0x05,
///                0x02, 0x03, 0x01, 0x00, 0x01];
/// let expected1  = BerObject::from_seq(vec![
///     BerObject::from_obj(
///         BerObjectContent::ContextSpecific(BerTag(0),
///             Some(Box::new(BerObject::from_obj(BerObjectContent::Enum(1)))))
///     ),
///     BerObject::from_int_slice(b"\x01\x00\x01"),
/// ]);
/// let expected2  = BerObject::from_seq(vec![
///     BerObject::from_obj(
///         BerObjectContent::ContextSpecific(BerTag(0), None),
///     ),
///     BerObject::from_int_slice(b"\x01\x00\x01"),
/// ]);
///
/// fn parse_optional_enum(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_optional!(i, parse_ber_enum)
/// }
/// fn parser(i:&[u8]) -> IResult<&[u8],BerObject> {
///     parse_der_sequence_defined!(i,
///         parse_optional_enum,
///         parse_ber_integer
///     )
/// };
///
/// assert_eq!(parser(&bytes1), Ok((empty, expected1)));
/// assert_eq!(parser(&bytes2), Ok((empty, expected2)));
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_optional(
    ($i:expr, $f:ident) => (
        alt_complete!(
            $i,
            do_parse!(
                content: call!($f) >>
                (
                    $crate::ber::BerObject::from_obj(
                        $crate::ber::BerObjectContent::ContextSpecific($crate::ber::BerTag(0) /* XXX */,Some(Box::new(content)))
                    )
                )
            ) |
            apply!($crate::ber::parse_ber_explicit_failed,$crate::ber::BerTag(0) /* XXX */)
        )
    )
);

/// Parse a constructed DER element
///
/// Read a constructed DER element (sequence or set, typically) using the provided functions.
/// This is generally used to build a struct from a DER sequence.
///
/// The returned object is a tuple containing a [`BerObjectHeader`](struct.BerObjectHeader.html)
/// and the object returned by the subparser.
///
/// To ensure the subparser consumes all bytes from the constructed object, add the `empty!()`
/// subparser as the last parsing item.
///
/// To verify the tag of the constructed element, use the `TAG` version, for ex
/// `parse_der_struct!(i, TAG DerTag::Sequence, parse_der_integer)`
///
/// Similar to [`parse_der_sequence_defined`](macro.parse_der_sequence_defined.html), but using the
/// `do_parse` macro from nom.
/// This allows declaring variables, and running code at the end.
///
/// # Examples
///
/// Basic struct parsing (ignoring tag):
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// #[derive(Debug, PartialEq)]
/// struct MyStruct<'a>{
///     a: BerObject<'a>,
///     b: BerObject<'a>,
/// }
///
/// fn parse_struct01(i: &[u8]) -> IResult<&[u8],(BerObjectHeader,MyStruct)> {
///     parse_der_struct!(
///         i,
///         a: parse_ber_integer >>
///         b: parse_ber_integer >>
///            empty!() >>
///         ( MyStruct{ a: a, b: b } )
///     )
/// }
///
/// let bytes = [ 0x30, 0x0a,
///               0x02, 0x03, 0x01, 0x00, 0x01,
///               0x02, 0x03, 0x01, 0x00, 0x00,
/// ];
/// let empty = &b""[..];
/// let expected = (
///     BerObjectHeader{
///         class: 0,
///         structured: 1,
///         tag: BerTag::Sequence,
///         len: 0xa,
///     },
///     MyStruct {
///         a: BerObject::from_int_slice(b"\x01\x00\x01"),
///         b: BerObject::from_int_slice(b"\x01\x00\x00"),
///     }
/// );
/// let res = parse_struct01(&bytes);
/// assert_eq!(res, Ok((empty, expected)));
/// # }
/// ```
///
/// To check the expected tag, use the `TAG <tagname>` variant:
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// # use der_parser::ber::*;
/// # use nom::{IResult,Err,ErrorKind};
/// # fn main() {
/// struct MyStruct<'a>{
///     a: BerObject<'a>,
///     b: BerObject<'a>,
/// }
///
/// fn parse_struct_with_tag(i: &[u8]) -> IResult<&[u8],(BerObjectHeader,MyStruct)> {
///     parse_der_struct!(
///         i,
///         TAG BerTag::Sequence,
///         a: parse_ber_integer >>
///         b: parse_ber_integer >>
///            empty!() >>
///         ( MyStruct{ a: a, b: b } )
///     )
/// }
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_struct(
    ($i:expr, TAG $tag:expr, $($rest:tt)*) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader|
                         hdr.structured == 1 && hdr.tag == $tag) >>
            res: flat_take!(hdr.len as usize, do_parse!( $($rest)* )) >>
            (hdr,res)
        )
    });
    ($i:expr, $($rest:tt)*) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader| hdr.structured == 1) >>
            res: flat_take!(hdr.len as usize, do_parse!( $($rest)* )) >>
            (hdr,res)
        )
    });
);

/// Parse a tagged DER element
///
/// Read a tagged DER element using the provided function.
///
/// The returned object is either the object returned by the subparser, or a nom error.
/// Unlike [`parse_der_explicit`](fn.parse_der_explicit.html) or
/// [`parse_der_implicit`](fn.parse_der_implicit.html), the returned values are *not* encapsulated
/// in a `BerObject` (they are directly returned, without the tag).
///
/// To specify the kind of tag, use the EXPLICIT or IMPLICIT keyword. If no keyword is specified,
/// the parsing is EXPLICIT by default.
///
/// When parsing IMPLICIT values, the third argument is a [`DerTag`](enum.DerTag.html) defining the
/// subtype of the object.
///
/// # Examples
///
/// The following parses `[2] INTEGER`:
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn parse_int_explicit(i:&[u8]) -> IResult<&[u8],u32> {
///     map_res!(
///         i,
///         parse_der_tagged!(EXPLICIT 2, parse_ber_integer),
///         |x: BerObject| x.as_u32()
///     )
/// }
/// let bytes = &[0xa2, 0x05, 0x02, 0x03, 0x01, 0x00, 0x01];
/// let res = parse_int_explicit(bytes);
/// match res {
///     Ok((rem,val)) => {
///         assert!(rem.is_empty());
///         assert_eq!(val, 0x10001);
///     },
///     _ => assert!(false)
/// }
/// # }
/// ```
///
/// The following parses `[2] IMPLICIT INTEGER`:
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// fn parse_int_implicit(i:&[u8]) -> IResult<&[u8],u32> {
///     map_res!(
///         i,
///         parse_der_tagged!(IMPLICIT 2, BerTag::Integer),
///         |x: BerObject| x.as_u32()
///     )
/// }
/// let bytes = &[0x82, 0x03, 0x01, 0x00, 0x01];
/// let res = parse_int_implicit(bytes);
/// match res {
///     Ok((rem,val)) => {
///         assert!(rem.is_empty());
///         assert_eq!(val, 0x10001);
///     },
///     _ => assert!(false)
/// }
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_tagged(
    ($i:expr, EXPLICIT $tag:expr, $f:ident) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader| hdr.tag.0 == $tag) >>
            res: flat_take!(hdr.len as usize, call!( $f )) >>
            (res)
        )
    });
    ($i:expr, EXPLICIT $tag:expr, $submac:ident!( $($args:tt)*)) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader| hdr.tag.0 == $tag) >>
            res: flat_take!(hdr.len as usize, $submac!( $($args)* )) >>
            (res)
        )
    });
    ($i:expr, IMPLICIT $tag:expr, $type:expr) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header,ber_read_element_content_as};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader| hdr.tag.0 == $tag) >>
            res: call!(ber_read_element_content_as, $type, hdr.len as usize, hdr.is_constructed(), 0) >>
            (BerObject::from_obj(res))
        )
    });
    ($i:expr, $tag:expr, $f:ident) => ( parse_der_tagged!($i, EXPLICIT $tag, $f) );
);

/// Parse an application DER element
///
/// Read an application DER element using the provided functions.
/// This is generally used to build a struct from a DER sequence.
///
/// The returned object is a tuple containing a [`BerObjectHeader`](struct.BerObjectHeader.html)
/// and the object returned by the subparser.
///
/// To ensure the subparser consumes all bytes from the constructed object, add the `empty!()`
/// subparser as the last parsing item.
///
/// # Examples
///
/// The following parses `[APPLICATION 2] INTEGER`:
///
/// ```rust
/// # #[macro_use] extern crate nom;
/// # #[macro_use] extern crate rusticata_macros;
/// # #[macro_use] extern crate der_parser;
/// use der_parser::ber::*;
/// use nom::{IResult,Err,ErrorKind};
///
/// # fn main() {
/// #[derive(Debug, PartialEq)]
/// struct SimpleStruct {
///     a: u32,
/// };
/// fn parse_app01(i:&[u8]) -> IResult<&[u8],(BerObjectHeader,SimpleStruct)> {
///     parse_der_application!(
///         i,
///         APPLICATION 2,
///         a: map_res!(parse_ber_integer,|x: BerObject| x.as_u32()) >>
///            empty!() >>
///         ( SimpleStruct{ a:a } )
///     )
/// }
/// let bytes = &[0x62, 0x05, 0x02, 0x03, 0x01, 0x00, 0x01];
/// let res = parse_app01(bytes);
/// match res {
///     Ok((rem,(hdr,app))) => {
///         assert!(rem.is_empty());
///         assert_eq!(hdr.tag, BerTag::Integer);
///         assert!(hdr.is_application());
///         assert_eq!(hdr.structured, 1);
///         assert_eq!(app, SimpleStruct{ a:0x10001 });
///     },
///     _ => assert!(false)
/// }
/// # }
/// ```
#[macro_export]
macro_rules! parse_der_application(
    ($i:expr, APPLICATION $tag:expr, $($rest:tt)*) => ({
        use $crate::ber::{BerObjectHeader,ber_read_element_header};
        do_parse!(
            $i,
            hdr: verify!(ber_read_element_header, |ref hdr: BerObjectHeader|
                         hdr.class == 0b01 && hdr.tag.0 == $tag) >>
            res: flat_take!(hdr.len as usize, do_parse!( $($rest)* )) >>
            (hdr,res)
        )
    });
    ($i:expr, $tag:expr, $($rest:tt)*) => ( parse_der_application!($i, $tag, $($rest)*) );
);
