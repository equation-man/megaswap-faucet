//! This is the program's entrypoint.
#![allow(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]
use pinocchio::{
    Address, AccountView, ProgramResult, error::ProgramError
};
use pinocchio_pubkey::declare_id;
use pinocchio_log::log;

pub mod config;
use config::*;

declare_id!("9uwR3ZyHXhnA2QvPDHtjg5ei3AT9VTzst6pbzj6eQjLn");
