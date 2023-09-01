use sha2::Sha256;
use hmac::{Hmac, Mac};



pub fn verify_signature(payload: Vec<u8> , signature: &str, secret: &str) -> bool {
 
     let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
     mac.update(payload.as_slice());

     let result =  mac.finalize().into_bytes();

     let computed_signature  = {
         let mut s = String::new();
         for byte in result.iter() {
             s.push_str(&format!("{:02x}", byte));
         }
         s
        };

    return format!("sha256={}", computed_signature) == signature;
 
}