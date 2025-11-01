/// Example: Efficient batch hashing with CUDA
///
/// Run with: cargo run --example batch_hashing

use ts3_sec_cuda_rs::hashers::CudaHasher;

fn main() {
    println!("🚀 CUDA Batch Hashing Example\n");

    // Initialize CUDA hasher
    let hasher = CudaHasher::new().expect("Failed to initialize CUDA");

    // Example 1: Hash your three messages
    println!("Example 1: Hash three messages");
    let messages = vec![b"123", b"abc", b"xyz"];

    let results = hasher.hash_messages_batch(
        &messages.iter().map(|&m| m as &[u8]).collect::<Vec<_>>()
    ).expect("Batch hashing failed");

    for (msg, hash) in messages.iter().zip(results.iter()) {
        let hash_hex = hash.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        println!("  SHA1({:?}) = {}", String::from_utf8_lossy(*msg), hash_hex);
    }

    // Example 2: Simulate TS3 counter searching (batch mode)
    println!("\nExample 2: Search for good counters (batch mode)");
    let public_key = "test_key";
    let start_counter = 1000;
    let batch_size = 10000;

    // Prepare batch of messages: public_key + counter
    let mut messages_vec: Vec<String> = Vec::new();
    for i in start_counter..start_counter + batch_size {
        messages_vec.push(format!("{}{}", public_key, i));
    }

    let messages_refs: Vec<&[u8]> = messages_vec.iter()
        .map(|s| s.as_bytes())
        .collect();

    println!("  Hashing {} messages in parallel...", batch_size);
    let start = std::time::Instant::now();
    let hashes = hasher.hash_messages_batch(&messages_refs)
        .expect("Batch hashing failed");
    let elapsed = start.elapsed();

    let hashrate = batch_size as f64 / elapsed.as_secs_f64();
    println!("  ✓ Completed in {:.3}s", elapsed.as_secs_f64());
    println!("  ✓ Hashrate: {:.2} MH/s", hashrate / 1_000_000.0);

    // Count trailing zeros (security level)
    let mut best_level = 0;
    let mut best_counter = start_counter;

    for (i, hash) in hashes.iter().enumerate() {
        let level = count_trailing_zero_bits(hash);
        if level > best_level {
            best_level = level;
            best_counter = start_counter + i;
        }
    }

    println!("  ✓ Best counter: {} (level {})", best_counter, best_level);
}

fn count_trailing_zero_bits(hash: &[u8]) -> u8 {
    let mut count = 0;
    for &byte in hash {
        if byte == 0 {
            count += 8;
        } else {
            count += byte.trailing_zeros() as u8;
            break;
        }
    }
    count
}
