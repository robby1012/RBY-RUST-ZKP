use num_bigint::BigUint;
use std::io::stdin;

pub mod zkp_auth {
    include!("./zkp_auth.rs");
}

use zkp_auth::{
    auth_client::AuthClient, AuthenticationAnswerRequest, AuthenticationChallengeRequest,
    RegisterRequest,
};
use zkp_chaum_pedersen::ZKP;

#[tokio::main]
async fn main() {
    let mut buf = String::new();
    let (alpha, beta, p, q) = ZKP::get_constants();
    let zkp = ZKP {
        alpha: alpha.clone(),
        beta: beta.clone(),
        p: p.clone(),
        q: q.clone(),
    };
    println!("[debug] alpha: {:?}\n",&alpha.to_bytes_be());
    println!("[debug] beta: {:?}\n",&beta.to_bytes_be());
    println!("[debug] p: {:?}\n",&p.to_bytes_be());
    println!("[debug] q: {:?}\n\n",&q.to_bytes_be());


    let mut client = AuthClient::connect("http://localhost:50051")
        .await
        .expect("could not connect to the server");
    println!("✅ Connected to the server");

    println!("Please provide the username:");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get the username from stdin");
    let username = buf.trim().to_string();
    buf.clear();

    println!("Please provide the password:");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get the username from stdin");
    let password = BigUint::from_bytes_be(buf.trim().as_bytes());
    buf.clear();

    let (y1, y2) = zkp.compute_pair(&password); 

    let request = RegisterRequest {
        user: username.clone(),
        y1: y1.to_bytes_be(),
        y2: y2.to_bytes_be(),
    };   
    println!("\n\n[debug] y1 (byte): {:?}\n",&y1.to_bytes_be());  
    println!("[debug] y2 (byte): {:?}\n",&y2.to_bytes_be());

    let _response = client
        .register(request)
        .await
        .expect("Could not register in server");  
    println!("[debug] Response: {:?}\n",&_response);

    println!("✅ Registration was successful");

    println!("Please provide the password (to login):");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get the username from stdin");
    let password = BigUint::from_bytes_be(buf.trim().as_bytes());
    buf.clear();

    let k = ZKP::generate_random_number_below(&q);
    let (r1, r2) = zkp.compute_pair(&k); 
    println!("=============================LOGIN==================================="); 
    println!("\n[debug] k (Secret Number): {:?}\n",&k); 
    println!("[debug] k (Secret Number) Byte: {:?}\n",&k.to_bytes_be()); 

    let request = AuthenticationChallengeRequest {
        user: username,
        r1: r1.to_bytes_be(),
        r2: r2.to_bytes_be(),
    }; 
    println!("[debug] r1 (byte): {:?}\n",&r1.to_bytes_be());  
    println!("[debug] r2 (byte): {:?}\n",&y2.to_bytes_be());

    let response = client
        .create_authentication_challenge(request)
        .await
        .expect("Could not request challenge to server")
        .into_inner();
    println!("[debug] response: {:?}\n",&response);

    let auth_id = response.auth_id;
    let c = BigUint::from_bytes_be(&response.c); 
    println!("[debug] c (Byte): {:?}\n",&c.to_bytes_be());

    let s = zkp.solve(&k, &c, &password);  
    println!("[debug] auth_id: {}\n",&auth_id);
    println!("[debug] s (byte): {:?}\n",&s.to_bytes_be());

    let request = AuthenticationAnswerRequest {
        auth_id,
        s: s.to_bytes_be(),
    };

    let response = client
        .verify_authentication(request)
        .await
        .expect("Could not verify authentication in server")
        .into_inner(); 
    println!("[debug] response: {:?}",&response);  

    println!("✅Logging successful! session_id: {}", response.session_id);
}
