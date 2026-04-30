use crate::protocol::pb::{CompressionType, FeatureEmbedding};

pub fn compress_embedding(
    vec: &[f32],
    method: CompressionType,
) -> FeatureEmbedding {
    match method {
        CompressionType::Binary => unimplemented!(),
        _ => {
            let bytes = vec_to_bytes(vec);
            FeatureEmbedding {
                model_hash: 0,
                compression: method as i32,
                signature: bytes,
            }
        },
    }
}

fn vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}
