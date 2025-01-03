use std::sync::Arc;

use async_trait::async_trait;
#[cfg(any(feature = "testing", test))]
use mockall::automock;
use papyrus_proc_macros::handle_response_variants;
use serde::{Deserialize, Serialize};
use starknet_sequencer_infra::component_client::{
    ClientError,
    LocalComponentClient,
    RemoteComponentClient,
};
use starknet_sequencer_infra::component_definitions::{
    ComponentClient,
    ComponentRequestAndResponseSender,
};
use starknet_state_sync_types::state_sync_types::SyncBlock;
use thiserror::Error;

use crate::batcher_types::{
    BatcherResult,
    DecisionReachedInput,
    DecisionReachedResponse,
    GetHeightResponse,
    GetProposalContentInput,
    GetProposalContentResponse,
    ProposeBlockInput,
    SendProposalContentInput,
    SendProposalContentResponse,
    StartHeightInput,
    ValidateBlockInput,
};
use crate::errors::BatcherError;

pub type LocalBatcherClient = LocalComponentClient<BatcherRequest, BatcherResponse>;
pub type RemoteBatcherClient = RemoteComponentClient<BatcherRequest, BatcherResponse>;
pub type BatcherClientResult<T> = Result<T, BatcherClientError>;
pub type BatcherRequestAndResponseSender =
    ComponentRequestAndResponseSender<BatcherRequest, BatcherResponse>;
pub type SharedBatcherClient = Arc<dyn BatcherClient>;

/// Serves as the batcher's shared interface. Requires `Send + Sync` to allow transferring and
/// sharing resources (inputs, futures) across threads.
#[cfg_attr(any(test, feature = "testing"), automock)]
#[async_trait]
pub trait BatcherClient: Send + Sync {
    /// Starts the process of building a proposal.
    async fn propose_block(&self, input: ProposeBlockInput) -> BatcherClientResult<()>;
    /// Gets the first height that is not written in the storage yet.
    async fn get_height(&self) -> BatcherClientResult<GetHeightResponse>;
    /// Gets the next available content from the proposal stream (only relevant when building a
    /// proposal).
    async fn get_proposal_content(
        &self,
        input: GetProposalContentInput,
    ) -> BatcherClientResult<GetProposalContentResponse>;
    /// Starts the process of validating a proposal.
    async fn validate_block(&self, input: ValidateBlockInput) -> BatcherClientResult<()>;
    /// Sends the content of a proposal. Only relevant when validating a proposal.
    /// Note:
    ///   * This call can be blocking if the batcher has too many unprocessed transactions.
    ///   * The next send might receive an `InvalidProposal` response for the previous send.
    ///   * If this marks the end of the content, i.e. `SendProposalContent::Finish` is received,
    ///     the batcher will block until the proposal has finished processing before responding.
    async fn send_proposal_content(
        &self,
        input: SendProposalContentInput,
    ) -> BatcherClientResult<SendProposalContentResponse>;
    /// Starts the process of a new height.
    /// From this point onwards, the batcher will accept requests only for proposals associated
    /// with this height.
    async fn start_height(&self, input: StartHeightInput) -> BatcherClientResult<()>;
    /// Adds a block from the state sync. Updates the batcher's state and commits the
    /// transactions to the mempool.
    async fn add_sync_block(&self, sync_block: SyncBlock) -> BatcherClientResult<()>;
    /// Notifies the batcher that a decision has been reached.
    /// This closes the process of the given height, and the accepted proposal is committed.
    async fn decision_reached(
        &self,
        input: DecisionReachedInput,
    ) -> BatcherClientResult<DecisionReachedResponse>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BatcherRequest {
    ProposeBlock(ProposeBlockInput),
    GetProposalContent(GetProposalContentInput),
    ValidateBlock(ValidateBlockInput),
    SendProposalContent(SendProposalContentInput),
    StartHeight(StartHeightInput),
    GetCurrentHeight,
    DecisionReached(DecisionReachedInput),
    AddSyncBlock(SyncBlock),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BatcherResponse {
    ProposeBlock(BatcherResult<()>),
    GetCurrentHeight(BatcherResult<GetHeightResponse>),
    GetProposalContent(BatcherResult<GetProposalContentResponse>),
    ValidateBlock(BatcherResult<()>),
    SendProposalContent(BatcherResult<SendProposalContentResponse>),
    StartHeight(BatcherResult<()>),
    DecisionReached(BatcherResult<DecisionReachedResponse>),
    AddSyncBlock(BatcherResult<()>),
}

#[derive(Clone, Debug, Error)]
pub enum BatcherClientError {
    #[error(transparent)]
    ClientError(#[from] ClientError),
    #[error(transparent)]
    BatcherError(#[from] BatcherError),
}

#[async_trait]
impl<ComponentClientType> BatcherClient for ComponentClientType
where
    ComponentClientType: Send + Sync + ComponentClient<BatcherRequest, BatcherResponse>,
{
    async fn propose_block(&self, input: ProposeBlockInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::ProposeBlock(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, ProposeBlock, BatcherClientError, BatcherError)
    }

    async fn get_proposal_content(
        &self,
        input: GetProposalContentInput,
    ) -> BatcherClientResult<GetProposalContentResponse> {
        let request = BatcherRequest::GetProposalContent(input);
        let response = self.send(request).await;
        handle_response_variants!(
            BatcherResponse,
            GetProposalContent,
            BatcherClientError,
            BatcherError
        )
    }

    async fn validate_block(&self, input: ValidateBlockInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::ValidateBlock(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, ValidateBlock, BatcherClientError, BatcherError)
    }

    async fn send_proposal_content(
        &self,
        input: SendProposalContentInput,
    ) -> BatcherClientResult<SendProposalContentResponse> {
        let request = BatcherRequest::SendProposalContent(input);
        let response = self.send(request).await;
        handle_response_variants!(
            BatcherResponse,
            SendProposalContent,
            BatcherClientError,
            BatcherError
        )
    }

    async fn start_height(&self, input: StartHeightInput) -> BatcherClientResult<()> {
        let request = BatcherRequest::StartHeight(input);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, StartHeight, BatcherClientError, BatcherError)
    }

    async fn get_height(&self) -> BatcherClientResult<GetHeightResponse> {
        let request = BatcherRequest::GetCurrentHeight;
        let response = self.send(request).await;
        handle_response_variants!(
            BatcherResponse,
            GetCurrentHeight,
            BatcherClientError,
            BatcherError
        )
    }

    async fn decision_reached(
        &self,
        input: DecisionReachedInput,
    ) -> BatcherClientResult<DecisionReachedResponse> {
        let request = BatcherRequest::DecisionReached(input);
        let response = self.send(request).await;
        handle_response_variants!(
            BatcherResponse,
            DecisionReached,
            BatcherClientError,
            BatcherError
        )
    }

    async fn add_sync_block(&self, sync_block: SyncBlock) -> BatcherClientResult<()> {
        let request = BatcherRequest::AddSyncBlock(sync_block);
        let response = self.send(request).await;
        handle_response_variants!(BatcherResponse, AddSyncBlock, BatcherClientError, BatcherError)
    }
}
