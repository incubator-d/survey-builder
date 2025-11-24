//! Interfaces d'entrée - Adaptateurs pour différentes interfaces
//! 
//! Contient les adaptateurs pour CLI, Web, API, etc.

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "web")]
pub mod web;

