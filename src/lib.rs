#[cfg(test)]
mod tests {
    use bitstream_io::read::BitRead;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_bitstream_unary0() {
        let source = [0b01000000_u8];
        let mut bits = bitstream_io::read::BitReader::endian(&source[..], bitstream_io::BigEndian);
        println!("{:?}", bits.read_unary1());
        println!("{:?}", bits.read_unary1());
        println!("{:?}", bits.read_unary1());

        assert!(false);
    }
}
