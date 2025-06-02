use num_bigint::BigUint;
use std::io::stdin;

pub mod zkp_auth {
    include!("./zkp_auth.rs");
}

use zkp_auth::{
    auth_client::AuthClient, AuthenticationAnswerRequest, AuthenticationChallengeRequest
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

    let mut client: AuthClient<tonic::transport::Channel> = AuthClient::connect("http://localhost:50051")
        .await
        .expect("could not connect to the server");

    println!("✅ Connected to the server");

    println!("Enter Your username:");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get secret from stdin");
    let username = buf.trim().to_string();
    buf.clear();

    println!("Enter Your Secret:");
    stdin()
        .read_line(&mut buf)
        .expect("Could not get the username from stdin");
    let password = BigUint::from_bytes_be(buf.trim().as_bytes());
    buf.clear();

    let k = ZKP::generate_random_number_below(&q);
    let (r1, r2) = zkp.compute_pair(&k); 

    let request: AuthenticationChallengeRequest = AuthenticationChallengeRequest {
        user: username,
        r1: r1.to_bytes_be(),
        r2: r2.to_bytes_be(),
    }; 

    let response: zkp_auth::AuthenticationChallengeResponse = client
        .create_authentication_challenge(request)
        .await
        .expect("Could not request challenge to server")
        .into_inner();

    let auth_id = response.auth_id;
    let c = BigUint::from_bytes_be(&response.c); 

    let s = zkp.solve(&k, &c, &password); 

    let request: AuthenticationAnswerRequest = AuthenticationAnswerRequest {
        auth_id,
        s: s.to_bytes_be(),
    };

    let response: zkp_auth::AuthenticationAnswerResponse = client
        .verify_authentication(request)
        .await
        .expect("Could not verify authentication in server")
        .into_inner(); 

    println!("✅Logging successful! session_id: {}", response.session_id);
}