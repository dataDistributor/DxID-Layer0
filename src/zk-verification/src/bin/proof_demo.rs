use zk_verification::{generate_proof, verify_proof};

fn main() {
    let data = "Test Data";
    let proof = generate_proof(data);

    println!("Data: {}", data);
    println!("Generated Proof: {}", proof);

    if verify_proof(&proof, data) {
        println!("Proof is valid!");
    } else {
        println!("Proof is invalid!");
    }
}

