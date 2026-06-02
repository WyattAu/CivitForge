#![forbid(unsafe_code)]

pub mod agent;
pub mod ast;
pub mod embedding;
pub mod llm;
pub mod models;
pub mod qdrant;
pub mod rag;
pub mod rag_extended;
pub mod review;
pub mod treesitter;
pub mod vector;
pub mod vectordb;

pub use ast::engine::{AstNode, AstNodeType, ParseEngine};
pub use embedding::{EmbeddingVector, EmbeddingWorker};
pub use models::CodeEntity;
pub use rag::{InMemoryRagPipeline, RAGPipeline};
pub use vectordb::{QdrantVectorDbAdapter, VectorDb, VectorDbClient};
