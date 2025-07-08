use dotenv::dotenv;
use std::env;
use whisky::calculate_tx_hash;

use bowmore::{
    handler::{placeholder, sign_transaction},
    services::{
        self,
        bowmore_server::{Bowmore, BowmoreServer},
        TxHashResponse,
    },
};
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct BowmoreService {}

#[tonic::async_trait]
impl Bowmore for BowmoreService {
    async fn setup_vault_oracle(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - setup_vault_oracle {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn vault_deposit(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - vault_deposit {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn process_vault_deposit(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - process_vault_deposit {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn vault_withdrawal(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - vault_withdrawal {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn process_vault_withdrawal(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!(
            "Got a request - process_vault_withdrawal {:?}",
            request_result
        );

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn setup_swap_oracle(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - setup_swap_oracle {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn swap(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - swap {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn process_swap(
        &self,
        request: Request<services::Todo>,
    ) -> Result<Response<services::Todo>, Status> {
        let request_result = request.into_inner();
        println!("Got a request - process_swap {:?}", request_result);

        match placeholder().await {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(services::Todo {}))
    }

    async fn sign_transaction(
        &self,
        request: Request<services::SignTransactionRequest>,
    ) -> Result<Response<services::SignTransactionResponse>, Status> {
        println!("Got a request - sign_transaction");
        let request_result = request.into_inner();
        let reply = match sign_transaction::handler(request_result) {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(reply))
    }

    async fn calculate_tx_hash(
        &self,
        request: Request<services::CalculateTxHashRequest>,
    ) -> Result<Response<services::TxHashResponse>, Status> {
        println!("Got a request - calculate_tx_hash");
        let request_result = request.into_inner();
        let tx_hash = match calculate_tx_hash(&request_result.tx_hex) {
            Ok(value) => value,
            Err(e) => {
                return Err(Status::failed_precondition(e.to_string()));
            }
        };
        Ok(Response::new(TxHashResponse { tx_hash: tx_hash }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let port = env::var("PORT").unwrap_or_else(|_| "50051".to_string());
    let addr = format!("0.0.0.0:{}", port).parse()?;
    let transactions = BowmoreService::default();

    println!("Server listening on port {}...", port);
    Server::builder()
        .add_service(BowmoreServer::new(transactions))
        .serve(addr)
        .await?;
    Ok(())
}
