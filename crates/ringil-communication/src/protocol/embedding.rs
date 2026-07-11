use passerelle::ringil::swarm::v1::{CompressionType, FeatureEmbedding};

/// Compress high-dimensional AI embeddings for narrow-band transmission.
pub fn compress_embedding(
    vec: &[f32],
    method: CompressionType,
    model_id: &str,
) -> FeatureEmbedding {
    let bytes = match method {
        CompressionType::Binary => compress_binary(vec),
        CompressionType::None => vec_to_bytes(vec),
        _ => unimplemented!("advanced compression requires shared codebooks"),
    };

    FeatureEmbedding {
        model_hash: crc32fast::hash(model_id.as_bytes()),
        compression: method as i32,
        signature: bytes,
    }
}

/// Shrinks 512-dim vector from 2048 to 64 bytes.
fn compress_binary(vec: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vec.len().div_ceil(8));
    for chunk in vec.chunks(8) {
        let mut b = 0u8;
        for (i, &val) in chunk.iter().enumerate() {
            if val > 0.0 {
                b |= 1 << i;
            }
        }
        bytes.push(b);
    }
    bytes
}

fn vec_to_bytes(vec: &[f32]) -> Vec<u8> {
    vec.iter().flat_map(|f| f.to_le_bytes()).collect()
}
