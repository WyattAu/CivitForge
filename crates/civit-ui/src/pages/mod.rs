#![forbid(unsafe_code)]

pub mod explore;
pub mod home;
pub mod issue_detail;
pub mod issues;
pub mod login;
pub mod new_repo;
pub mod not_found;
pub mod orgs;
pub mod pipelines;
pub mod repo_detail;
pub mod repos;
pub mod search;
pub mod settings;
pub mod wiki;

pub use explore::*;
pub use home::*;
pub use issue_detail::*;
pub use issues::*;
pub use login::*;
pub use new_repo::*;
pub use not_found::*;
pub use orgs::*;
pub use pipelines::*;
pub use repo_detail::*;
pub use repos::*;
pub use search::*;
pub use settings::*;
pub use wiki::*;
