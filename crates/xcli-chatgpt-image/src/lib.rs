//! ChatGPT image generation helpers for x-cli.
//!
//! This crate currently exposes shared constants used by ChatGPT image
//! automation integrations. Keeping a real library target here is required
//! because the crate is listed as a workspace member.

pub const DEFAULT_BRIDGE_URL: &str = "http://127.0.0.1:10086";
pub const SESSION_NAME: &str = "chatgpt-image-cli";
pub const CHATGPT_IMAGES_URL: &str = "https://chatgpt.com/images/";
