#[cfg(test)]
mod types;

#[cfg(test)]
mod tests {
    use crate::types::{
        TestArrayOfArrayOfString, TestArrayOfBytesLength2, TestArrayOfMaps, TestArrayOfString,
        TestBool, TestBytes, TestBytesLength0, TestBytesLength32, TestBytesLength65536,
        TestEmptyStruct, TestEnum, TestInt8, TestInt16, TestInt32, TestInt64, TestMultipleFields,
        TestNestedArray, TestNestedMap, TestNullable, TestOptional, TestOptionalNullable,
        TestString, TestTaggedEnum, TestUint8, TestUint16, TestUint32, TestUint64, r#if, r#while,
    };

    #[test]
    fn test_bool() {
        let data1 = TestBool { foo: false };
        let data2 = TestBool { foo: true };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0xc2]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xc3]);

        assert_eq!(
            rmp_serde::from_slice::<TestBool>(&[0x81, 0x00, 0xc2]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestBool>(&[0x81, 0x00, 0xc3]).unwrap(),
            data2,
        );
    }

    #[test]
    fn test_uint8() {
        let data1 = TestUint8 { foo: 0 };
        let data2 = TestUint8 { foo: 0xff };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0x00]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xcc, 0xff]);

        assert_eq!(
            rmp_serde::from_slice::<TestUint8>(&[0x81, 0x00, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestUint8>(&[0x81, 0x00, 0xcc, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestUint8>(&[0x81, 0x00, 0xcc, 0xff]).unwrap(),
            data2,
        );

        assert!(rmp_serde::from_slice::<TestUint8>(&[0x81, 0x00, 0xcd, 0xff, 0xff]).is_err());
    }

    #[test]
    fn test_int8() {
        let data1 = TestInt8 { foo: 0x7f };
        let data2 = TestInt8 { foo: -0x80 };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0x7f]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xd0, 0x80]);

        assert_eq!(
            rmp_serde::from_slice::<TestInt8>(&[0x81, 0x00, 0x7f]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestInt8>(&[0x81, 0x00, 0xd0, 0x80]).unwrap(),
            data2,
        );
    }

    #[test]
    fn test_uint16() {
        let data1 = TestUint16 { foo: 0 };
        let data2 = TestUint16 { foo: 0xffff };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0x00]);
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xcd, 0xff, 0xff],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestUint16>(&[0x81, 0x00, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestUint16>(&[0x81, 0x00, 0xcd, 0xff, 0xff]).unwrap(),
            data2,
        );
    }

    #[test]
    fn test_int16() {
        let data1 = TestInt16 { foo: 0x7fff };
        let data2 = TestInt16 { foo: -0x8000 };

        assert_eq!(
            rmp_serde::to_vec(&data1).unwrap(),
            [0x81, 0x00, 0xcd, 0x7f, 0xff],
        );
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xd1, 0x80, 0x00],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestInt16>(&[0x81, 0x00, 0xcd, 0x7f, 0xff]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestInt16>(&[0x81, 0x00, 0xd1, 0x80, 0x00]).unwrap(),
            data2,
        );
    }

    #[test]
    fn test_uint32() {
        let data1 = TestUint32 { foo: 0 };
        let data2 = TestUint32 { foo: 0xffffffff };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0x00]);
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xce, 0xff, 0xff, 0xff, 0xff],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestUint32>(&[0x81, 0x00, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestUint32>(&[0x81, 0x00, 0xce, 0xff, 0xff, 0xff, 0xff])
                .unwrap(),
            data2
        );
    }

    #[test]
    fn test_int32() {
        let data1 = TestInt32 { foo: 0x7fffffff };
        let data2 = TestInt32 { foo: -0x80000000 };

        assert_eq!(
            rmp_serde::to_vec(&data1).unwrap(),
            [0x81, 0x00, 0xce, 0x7f, 0xff, 0xff, 0xff],
        );
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xd2, 0x80, 0x00, 0x00, 0x00],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestInt32>(&[0x81, 0x00, 0xd2, 0x7f, 0xff, 0xff, 0xff])
                .unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestInt32>(&[0x81, 0x00, 0xd2, 0x80, 0x00, 0x00, 0x00])
                .unwrap(),
            data2,
        );
    }

    #[test]
    fn test_uint64() {
        let data1 = TestUint64 { foo: 0 };
        let data2 = TestUint64 {
            foo: 0xffffffffffffffff,
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0x00]);
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [
                0x81, 0x00, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestUint64>(&[0x81, 0x00, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestUint64>(&[
                0x81, 0x00, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ])
            .unwrap(),
            data2,
        );
    }

    #[test]
    fn test_int64() {
        let data1 = TestInt64 {
            foo: 0x7fffffffffffffff,
        };
        let data2 = TestInt64 {
            foo: -0x8000000000000000,
        };

        assert_eq!(
            rmp_serde::to_vec(&data1).unwrap(),
            [
                0x81, 0x00, 0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ],
        );
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [
                0x81, 0x00, 0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestInt64>(&[
                0x81, 0x00, 0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ])
            .unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestInt64>(&[
                0x81, 0x00, 0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ])
            .unwrap(),
            data2,
        );
    }

    #[test]
    fn test_string() {
        let data1 = TestString { foo: String::new() };
        let data2 = TestString {
            foo: String::from("foo"),
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0xa0]);
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xa3, 0x66, 0x6f, 0x6f],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestString>(&[0x81, 0x00, 0xa0]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestString>(&[0x81, 0x00, 0xa3, 0x66, 0x6f, 0x6f]).unwrap(),
            data2,
        );

        assert!(
            rmp_serde::from_slice::<TestString>(&[0x81, 0x00, 0xc4, 0x03, 0x66, 0x6f, 0x6f])
                .is_err(),
        );
    }

    #[test]
    fn test_bytes() {
        let data1 = TestBytes {
            foo: Vec::new().into(),
        };
        let data2 = TestBytes {
            foo: vec![0x00, 0x66, 0x6f, 0x6f].into(),
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0xc4, 0x00]);
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x81, 0x00, 0xc4, 0x04, 0x00, 0x66, 0x6f, 0x6f],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestBytes>(&[0x81, 0x00, 0xc4, 0x00]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestBytes>(&[0x81, 0x00, 0xc4, 0x04, 0x00, 0x66, 0x6f, 0x6f])
                .unwrap(),
            data2,
        );
        assert!(rmp_serde::from_slice::<TestBytes>(&[0x81, 0x00, 0xa0]).is_err());
    }

    #[test]
    fn test_bytes_length_0() {
        let data = TestBytesLength0 { foo: [].into() };

        assert_eq!(rmp_serde::to_vec(&data).unwrap(), [0x81, 0x00, 0xc4, 0x00]);

        assert_eq!(
            rmp_serde::from_slice::<TestBytesLength0>(&[0x81, 0x00, 0xc4, 0x00]).unwrap(),
            data,
        );
        assert!(
            rmp_serde::from_slice::<TestBytesLength0>(&[0x81, 0x00, 0xc4, 0x01, 0x01]).is_err(),
        );
    }

    #[test]
    fn test_bytes_length_32() {
        let data = TestBytesLength32 {
            foo: [0x01; 32].into(),
        };

        let mut encoded = [0x01; 36];
        encoded[0] = 0x81;
        encoded[1] = 0x00;
        encoded[2] = 0xc4;
        encoded[3] = 0x20;

        assert_eq!(rmp_serde::to_vec(&data).unwrap(), encoded);

        assert_eq!(
            rmp_serde::from_slice::<TestBytesLength32>(&encoded).unwrap(),
            data,
        );
        assert!(
            rmp_serde::from_slice::<TestBytesLength32>(&[0x81, 0x00, 0xc4, 0x01, 0x01]).is_err(),
        );
    }

    #[test]
    fn test_bytes_length_65536() {
        let data = TestBytesLength65536 {
            foo: [0x02; 65536].into(),
        };

        let mut encoded = [0x02; 65543];
        encoded[0] = 0x81;
        encoded[1] = 0x00;
        encoded[2] = 0xc6;
        encoded[3] = 0x00;
        encoded[4] = 0x01;
        encoded[5] = 0x00;
        encoded[6] = 0x00;

        assert_eq!(rmp_serde::to_vec(&data).unwrap(), encoded);

        assert_eq!(
            rmp_serde::from_slice::<TestBytesLength65536>(&encoded).unwrap(),
            data,
        );
        assert!(
            rmp_serde::from_slice::<TestBytesLength65536>(&[0x81, 0x00, 0xc4, 0x01, 0x01]).is_err(),
        );
    }

    #[test]
    fn test_array_of_bytes_length_2() {
        let data = TestArrayOfBytesLength2 {
            foo: [[0x01, 0x02].into(), [0x03, 0x04].into()].into(),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [
                0x81, 0x00, 0x92, 0xc4, 0x02, 0x01, 0x02, 0xc4, 0x02, 0x03, 0x04,
            ],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestArrayOfBytesLength2>(&[
                0x81, 0x00, 0x92, 0xc4, 0x02, 0x01, 0x02, 0xc4, 0x02, 0x03, 0x04,
            ],)
            .unwrap(),
            data,
        );
    }

    #[test]
    fn test_array_of_string() {
        let data = TestArrayOfString {
            foo: [String::new(), String::from("A")].into(),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [0x81, 0x00, 0x92, 0xa0, 0xa1, 0x41],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestArrayOfString>(&[0x81, 0x00, 0x92, 0xa0, 0xa1, 0x41])
                .unwrap(),
            data,
        );
    }

    #[test]
    fn test_array_of_array_of_string() {
        let data = TestArrayOfArrayOfString {
            foo: [[String::new(), String::from("A")].into()].into(),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [0x81, 0x00, 0x91, 0x92, 0xa0, 0xa1, 0x41],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestArrayOfArrayOfString>(&[
                0x81, 0x00, 0x91, 0x92, 0xa0, 0xa1, 0x41,
            ])
            .unwrap(),
            data,
        );
    }

    #[test]
    fn test_optional() {
        let data1 = TestOptional { foo: None };
        let data2 = TestOptional {
            foo: Some(String::new()),
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x80]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xa0]);

        assert_eq!(
            rmp_serde::from_slice::<TestOptional>(&[0x80]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestOptional>(&[0x81, 0x00, 0xa0]).unwrap(),
            data2,
        );

        assert!(rmp_serde::from_slice::<TestOptional>(&[0x81, 0x00, 0xc0]).is_err());
    }

    #[test]
    fn test_nullable() {
        let data1 = TestNullable { foo: None };
        let data2 = TestNullable {
            foo: Some(String::new()),
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x81, 0x00, 0xc0]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xa0]);

        assert_eq!(
            rmp_serde::from_slice::<TestNullable>(&[0x81, 0x00, 0xc0]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestNullable>(&[0x81, 0x00, 0xa0]).unwrap(),
            data2,
        );

        assert!(rmp_serde::from_slice::<TestNullable>(&[0x80]).is_err());
    }

    #[test]
    fn test_optional_nullable() {
        let data1 = TestOptionalNullable { foo: None };
        let data2 = TestOptionalNullable { foo: Some(None) };
        let data3 = TestOptionalNullable {
            foo: Some(Some(String::new())),
        };

        assert_eq!(rmp_serde::to_vec(&data1).unwrap(), [0x80]);
        assert_eq!(rmp_serde::to_vec(&data2).unwrap(), [0x81, 0x00, 0xc0]);
        assert_eq!(rmp_serde::to_vec(&data3).unwrap(), [0x81, 0x00, 0xa0]);

        assert_eq!(
            rmp_serde::from_slice::<TestOptionalNullable>(&[0x80]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestOptionalNullable>(&[0x81, 0x00, 0xc0]).unwrap(),
            data2,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestOptionalNullable>(&[0x81, 0x00, 0xa0]).unwrap(),
            data3,
        );
    }

    #[test]
    fn test_multiple_fields() {
        let data = TestMultipleFields {
            foo: String::from("a"),
            bar: String::from("b"),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [0x82, 0x00, 0xa1, 0x61, 0x02, 0xa1, 0x62],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestMultipleFields>(&[
                0x82, 0x00, 0xa1, 0x61, 0x02, 0xa1, 0x62,
            ])
            .unwrap(),
            data,
        );
    }

    #[test]
    fn test_nested_map() {
        let data = TestNestedMap {
            foo: TestString { foo: String::new() },
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [0x81, 0x00, 0x81, 0x00, 0xa0],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestNestedMap>(&[0x81, 0x00, 0x81, 0x00, 0xa0]).unwrap(),
            data,
        );
    }

    #[test]
    fn test_array_of_maps() {
        let data = TestArrayOfMaps {
            foo: [TestString { foo: String::new() }].into(),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [0x81, 0x00, 0x91, 0x81, 0x00, 0xa0],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestArrayOfMaps>(&[0x81, 0x00, 0x91, 0x81, 0x00, 0xa0])
                .unwrap(),
            data,
        );
    }

    #[test]
    fn test_nested_array() {
        let data = TestNestedArray {
            foo: [
                [[1, 2].into(), [3].into()].into(),
                [].into(),
                [[].into()].into(),
            ]
            .into(),
        };

        assert_eq!(
            rmp_serde::to_vec(&data).unwrap(),
            [
                0x81, 0x00, 0x93, 0x92, 0x92, 0x01, 0x02, 0x91, 0x03, 0x90, 0x91, 0x90,
            ],
        );
        assert_eq!(
            rmp_serde::from_slice::<TestNestedArray>(&[
                0x81, 0x00, 0x93, 0x92, 0x92, 0x01, 0x02, 0x91, 0x03, 0x90, 0x91, 0x90,
            ],)
            .unwrap(),
            data,
        );
    }

    #[test]
    fn test_enum() {
        assert_eq!(rmp_serde::to_vec(&TestEnum::Foo).unwrap(), [0x00]);
        assert_eq!(rmp_serde::to_vec(&TestEnum::Bar).unwrap(), [0x01]);
        assert_eq!(
            rmp_serde::from_slice::<TestEnum>(&[0x00]).unwrap(),
            TestEnum::Foo,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestEnum>(&[0x01]).unwrap(),
            TestEnum::Bar,
        );
    }

    #[test]
    fn test_rust_keywords() {
        let _ = r#if { r#else: 0 };
        let _ = r#while::r#for;
    }

    #[test]
    fn test_empty_struct() {
        let data = TestEmptyStruct {};

        assert_eq!(rmp_serde::to_vec(&data).unwrap(), [0x80]);
        assert_eq!(
            rmp_serde::from_slice::<TestEmptyStruct>(&[0x80]).unwrap(),
            data,
        );
    }

    #[test]
    fn test_tagged_enum() {
        let data1 = TestTaggedEnum::A(TestBool { foo: true });
        let data2 = TestTaggedEnum::B(TestUint8 { foo: 127 });

        assert_eq!(
            rmp_serde::to_vec(&data1).unwrap(),
            [0x92, 0x00, 0x81, 0x00, 0xc3],
        );
        assert_eq!(
            rmp_serde::to_vec(&data2).unwrap(),
            [0x92, 0x01, 0x81, 0x00, 0x7f],
        );

        assert_eq!(
            rmp_serde::from_slice::<TestTaggedEnum>(&[0x92, 0x00, 0x81, 0x00, 0xc3]).unwrap(),
            data1,
        );
        assert_eq!(
            rmp_serde::from_slice::<TestTaggedEnum>(&[0x92, 0x01, 0x81, 0x00, 0x7f]).unwrap(),
            data2,
        );

        assert!(rmp_serde::from_slice::<TestTaggedEnum>(&[0x92, 0x02, 0x80]).is_err());
        assert!(rmp_serde::from_slice::<TestTaggedEnum>(&[0x91, 0x00]).is_err());
        assert!(
            rmp_serde::from_slice::<TestTaggedEnum>(&[0x93, 0x00, 0x81, 0x00, 0xc3, 0x00]).is_err()
        );
        assert!(rmp_serde::from_slice::<TestTaggedEnum>(&[0x92, 0x01, 0x81, 0x00, 0xc3]).is_err());
    }
}
