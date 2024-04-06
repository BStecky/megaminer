use std::{
    io::{stdout, Write},
    time::Duration,
};

use solana_client::{
    client_error::{ClientError, ClientErrorKind, Result as ClientResult},
    nonblocking::rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use solana_program::instruction::Instruction;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel},
    signature::{Signature, Signer},
    transaction::Transaction,
};
use solana_transaction_status::{TransactionConfirmationStatus, UiTransactionEncoding};

use crate::Miner;

const RPC_RETRIES: usize = 1;

impl Miner {
    pub async fn send_and_confirm(
        &self,
        ixs: &[Instruction],
        skip_confirm: bool,
    ) -> ClientResult<Signature> {

        let send_attempts = 100; 
        let signer = self.signer();
        let client = RpcClient::new_with_commitment(self.cluster.clone(), CommitmentConfig::confirmed());

        // Build transaction
        let (hash, slot) = client
            .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
            .await
            .unwrap();
        let send_cfg = RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: Some(CommitmentLevel::Confirmed),
            encoding: Some(UiTransactionEncoding::Base64),
            max_retries: Some(RPC_RETRIES),
            min_context_slot: Some(slot),
        };
        let mut tx = Transaction::new_with_payer(ixs, Some(&signer.pubkey()));
        tx.sign(&[&signer], hash);

        // Initialize a variable to store the last signature
        let mut last_sig: Option<Signature> = None;

        // Attempt to send the transaction multiple times
        for _ in 0..send_attempts {
            match client.send_transaction_with_config(&tx, send_cfg.clone()).await {
                Ok(sig) => {
                    println!("Transaction submitted: {:?}", sig);
                    last_sig = Some(sig); // Update the last known signature
                },
                Err(err) => {
                    println!("Error submitting transaction: {:?}", err);
                    // Optionally, you could decide to break out of the loop on certain errors
                },
            }
            // Consider adding a short delay between attempts to avoid overwhelming the RPC server
            // tokio::time::sleep(Duration::from_millis(333)).await;
        }

        // Return the last known signature or an error if no attempts were successful
        match last_sig {
            Some(sig) => Ok(sig),
            None => Err(ClientError {
                request: None,
                kind: ClientErrorKind::Custom("Failed to submit transaction after multiple attempts".into()),
            }),
        }
    }
}
