//! This bundles all sub modules necessary for OCP.

#[macro_use]
// I don't know why this is necessary.. this is the legacy way..
// since macros can be called like a module path
extern crate neli;

#[macro_use]
extern crate log;

pub mod ocp_core;
pub mod ocp_properties;
