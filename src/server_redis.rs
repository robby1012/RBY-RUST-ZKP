use std::{collections::HashMap, sync::Mutex};

use num_bigint::BigUint;
use tonic::{transport::Server, Code, Request, Response, Status};
use serde::{Serialize, Deserialize};
use redis::{Client, Commands, Connection, RedisResult, RedisError};
use bincode;

use zkp_chaum_pedersen::ZKP;

pub mod zkp_auth {
    include!("./zkp_auth.rs");
}

use zkp_auth::{
    auth_server::{Auth, AuthServer},
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};

#[derive(Debug, Default)]
pub struct AuthImpl {
    pub user_info: Mutex<HashMap<String, UserInfo>>,
    pub auth_id_to_user: Mutex<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserInfo {
    // registration
    pub user_name: String,
    pub y1: BigUint,
    pub y2: BigUint,
    // authorization
    pub r1: BigUint,
    pub r2: BigUint,
    // verification
    pub c: BigUint,
    pub s: BigUint,
    pub session_id: String,
    pub auth_id: String
}

#[tonic::async_trait]
impl Auth for AuthImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = request.into_inner();

        let user_name = request.user;
        println!("Processing Registration username: {:?}", user_name);

        let user_info = UserInfo {
            user_name: user_name.clone(),
            y1: BigUint::from_bytes_be(&request.y1),
            y2: BigUint::from_bytes_be(&request.y2),
            ..Default::default()
        };

        println!("y1: {:?}",BigUint::from_bytes_be(&request.y1));
        println!("y2: {:?}",BigUint::from_bytes_be(&request.y2));

        // save data to redis
        save_data(user_info).expect("error saving data");

        println!("Successful Registration username ✅: {:?}", user_name);
        Ok(Response::new(RegisterResponse {}))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {

        let request: AuthenticationChallengeRequest = request.into_inner();

        let user_name: String = request.user;
        println!("Processing Challenge Request username: {:?}", user_name);

        if let Ok(Some(mut user_info)) = get_user_by_name(&user_name) {
            let (_, _, _, q) = ZKP::get_constants();
            let c: BigUint = ZKP::generate_random_number_below(&q);
            let auth_id: String = ZKP::generate_random_string(64);

            user_info.c = c.clone();
            user_info.r1 = BigUint::from_bytes_be(&request.r1);
            user_info.r2 = BigUint::from_bytes_be(&request.r2);

            save_data(user_info).expect("error saving data");

            let auth_id_to_user: &mut std::sync::MutexGuard<'_, HashMap<String, String>> = &mut self.auth_id_to_user.lock().unwrap();
            auth_id_to_user.insert(auth_id.clone(), user_name.clone());

            println!("✅ Successful Challenge Request username: {:?}", user_name);
            
            Ok(Response::new(AuthenticationChallengeResponse {
                auth_id,
                c: c.to_bytes_be(),
            }))
        } else {
            Err(Status::new(
                Code::NotFound,
                format!("User {} not found in database", &user_name),
            ))
        }
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        let request = request.into_inner();

        let auth_id = request.auth_id;
        println!("Processing Challenge Solution auth_id: {:?}", auth_id);

        let auth_id_to_user_hashmap: &mut std::sync::MutexGuard<'_, HashMap<String, String>> = &mut self.auth_id_to_user.lock().unwrap();

        if let Some(user_name) = auth_id_to_user_hashmap.get(&auth_id) {

            let mut user_info = get_user_by_name("robby")
            .map_err(|e| tonic::Status::internal(format!("Redis error: {}", e)))?
            .ok_or_else(|| Status::not_found("User not found"))?;

            let (alpha, beta, p, q) = ZKP::get_constants();
            let zkp: ZKP = ZKP { alpha, beta, p, q };

            let s: BigUint = BigUint::from_bytes_be(&request.s);
            user_info.s = s;

            let verification = zkp.verify(
                &user_info.r1,
                &user_info.r2,
                &user_info.y1,
                &user_info.y2,
                &user_info.c,
                &user_info.s,
            );

            println!("user_info: {:?}",&user_info);

            if verification {
                let session_id: String = ZKP::generate_random_string(64);

                let user_auth_id: &str = &auth_id;
                let user_session_id: &str = &session_id;
                user_info.auth_id = user_auth_id.to_string();
                user_info.session_id = user_session_id.to_string();

                save_data(user_info).expect("error saving data");

                println!("✅ Correct Challenge Solution username: {:?}", user_name);
                println!("[Debug] Your Auth ID: {}, Session ID: {}",auth_id,session_id);

                Ok(Response::new(AuthenticationAnswerResponse { session_id }))
            } else {
                println!("❌ Wrong Challenge Solution username: {:?}", user_name);

                Err(Status::new(
                    Code::PermissionDenied,
                    format!("AuthId: {} bad solution to the challenge", auth_id),
                ))
            }
        } else {
            Err(Status::new(
                Code::NotFound,
                format!("AuthId: {} not found in database", auth_id),
            ))
        }
    }
}

pub fn save_data(user_info: UserInfo) -> RedisResult<()> {
    let redis_client: Client = redis::Client::open("redis://127.0.0.1/")?;
    let mut redis_con: Connection = redis_client.get_connection()?;

    let binary: Vec<u8> = bincode::serialize(&user_info)
        .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Bincode error", e.to_string())))?;

    redis_con.set("user:rsession", binary)?;
    redis_con.set(format!("username:{}", user_info.user_name), "user:rsession")?;

    Ok(()) // ✅ Explicit and correct return
}

pub fn get_all_users(con: &mut redis::Connection) -> RedisResult<Vec<UserInfo>> {
    // Step 1: Find all keys matching pattern
    let keys: Vec<String> = con.keys("user:*")?;

    // Step 2: Fetch and deserialize each value
    let mut users: Vec<UserInfo> = Vec::new();

    for key in keys {
        let bin: Vec<u8> = con.get(&key)?;

        let user: UserInfo = bincode::deserialize(&bin).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::TypeError,
                "bincode deserialization failed",
                e.to_string(),
            ))
        })?;

        users.push(user);
    }

    Ok(users)
}

pub fn get_user_by_name(user_name: &str) -> RedisResult<Option<UserInfo>> {

    let redis_client: Client = redis::Client::open("redis://127.0.0.1/")?;
    let mut redis_con: Connection = redis_client.get_connection()?;

    if redis_con.exists(format!("username:{}", user_name))? {

        let data_key: String = redis_con.get(format!("username:{}", user_name))?;
        let bin: Vec<u8> = redis_con.get(data_key)?;

        let user: UserInfo = bincode::deserialize(&bin).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::TypeError,
                "bincode deserialization failed",
                e.to_string(),
            ))
        })?;

        Ok(Some(user))
    } else {
        // Not found
        Ok(None)
    }
}


#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:50051".to_string();

    println!("✅ Running the server in {}", addr);

    let auth_impl = AuthImpl::default();

    Server::builder()
        .add_service(AuthServer::new(auth_impl))
        .serve(addr.parse().expect("could not convert address"))
        .await
        .unwrap();
}
