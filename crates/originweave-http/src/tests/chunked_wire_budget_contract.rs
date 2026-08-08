use crate::HttpClientPolicy;
use crate::exchange::{extend_chunked_wire, maximum_chunked_wire_bytes};

#[test]
fn default_chunked_wire_prefix_stays_below_eighteen_mib() {
    let maximum = maximum_chunked_wire_bytes(&HttpClientPolicy::strict_defaults());
    assert_eq!(maximum, 18_104_340);
    assert!(maximum < 18 * 1024 * 1024);
}

#[test]
fn chunked_wire_growth_is_rejected_before_the_buffer_can_exceed_its_budget() {
    let mut wire = vec![0_u8; 3];

    assert!(matches!(
        extend_chunked_wire(&mut wire, &[4], 3),
        Err(crate::HttpError::EncodedContentTooLarge {
            byte_count: 4,
            maximum_bytes: 3,
        })
    ));
    assert_eq!(wire, vec![0_u8; 3]);
}
