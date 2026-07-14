#![forbid(unsafe_code)]

pub mod types;
pub mod store;

pub use store::ServiceMeshStore;
pub use types::{
    ServiceMeshService, ServiceMeshRoute, ServiceStatus,
    CreateServiceRequest, UpdateServiceRequest,
    CreateRouteRequest, UpdateRouteRequest,
};
