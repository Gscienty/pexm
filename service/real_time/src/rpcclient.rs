tonic::include_proto!("dal");

use tonic::transport::Channel;

pub type DalClient = dal_service_client::DalServiceClient<Channel>;
