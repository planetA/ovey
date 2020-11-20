//! This bundles all sub modules necessary for OCP.

#[macro_use]
// I don't know why this is necessary.. this is legacy I thought..
// since macros can be called like a module path
extern crate neli;

use crate::ocp_core::{OCPRecData, build_nl_attr, Ocp};
use crate::ocp_properties::{OveyOperation, OveyAttribute};

pub mod ocp_core;
pub mod ocp_properties;
