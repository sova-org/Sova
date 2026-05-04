use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameTextId(pub u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_text_id_roundtrips_through_messagepack() {
        let id = FrameTextId(42);
        let bytes = rmp_serde::to_vec_named(&id).unwrap();
        let decoded: FrameTextId = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(id, decoded);
    }
}
