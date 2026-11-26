use mibz2::{
    bwt::{InverseBwt, Plain},
    runs,
    stream::{Event, Scanner, SliceSource},
    Kernels,
};
use std::{env, fs};
mod endpoint;
include!("index.rs");
include!("kernel.rs");
include!("driver.rs");
