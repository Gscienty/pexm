tonic::include_proto!("clsi");

use tonic::transport::Channel;

pub type ClsiClient = clsi_service_client::ClsiServiceClient<Channel>;
