fn split_by_parity(arr: &[u8; 256]) -> ([u8; 128], [u8; 128]) {
    let mut evens = [0u8; 128];
    let mut odds = [0u8; 128];

    for i in 0..128 {
        evens[i] = arr[i * 2];
        odds[i] = arr[i * 2 + 1];
    }

    (evens, odds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_parity() {
        let data: [u8; 256] = std::array::from_fn(|i| (i % 2) as u8);
        
        assert_eq!(split_by_parity(&data), ([0u8; 128], [1u8; 128]))
    }
}