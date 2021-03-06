/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This software may be used and distributed according to the terms of the
 * GNU General Public License version 2.
 */

#![deny(warnings)]
#![feature(process_exitcode_placeholder)]
#![feature(async_closure)]
use anyhow::Error;
use fbinit::FacebookInit;
use futures::future::{self, FutureExt};

use cmdlib::{args, helpers::block_execute};

mod blobstore;
#[macro_use]
mod graph;
mod parse_node;
mod progress;
mod sampling;
mod scrub;
mod setup;
mod sizing;
mod state;
mod tail;
mod validate;
mod walk;

#[fbinit::main]
fn main(fb: FacebookInit) -> Result<(), Error> {
    // FIXME: Investigate why some SQL queries kicked off by the walker take 30s or more.
    newfilenodes::disable_sql_timeouts();

    let app_name = "walker";
    let matches = setup::setup_toplevel_app(app_name).get_matches();
    let logger = args::init_logging(fb, &matches);

    let future = match matches.subcommand() {
        (setup::SCRUB, Some(sub_m)) => {
            scrub::scrub_objects(fb, logger.clone(), &matches, sub_m).boxed()
        }
        (setup::COMPRESSION_BENEFIT, Some(sub_m)) => {
            sizing::compression_benefit(fb, logger.clone(), &matches, sub_m).boxed()
        }
        (setup::VALIDATE, Some(sub_m)) => {
            validate::validate(fb, logger.clone(), &matches, sub_m).boxed()
        }
        _ => {
            future::err::<_, Error>(Error::msg("Invalid Arguments, pass --help for usage.")).boxed()
        }
    };

    block_execute(
        future,
        fb,
        app_name,
        &logger,
        &matches,
        cmdlib::monitoring::AliveService,
    )
}
