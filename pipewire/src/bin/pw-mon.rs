// Copyright 2020, Collabora Ltd.
// SPDX-License-Identifier: MIT

use anyhow::Result;
use pipewire as pw;
use signal::Signal;
use std::sync::Arc;

use pw::prelude::*;
use pw::registry::ObjectType;

fn monitor() -> Result<()> {
    let main_loop = pw::MainLoop::new()?;

    let main_loop_weak = main_loop.downgrade();
    let _sig_int = main_loop.add_signal_local(Signal::SIGINT, move || {
        if let Some(main_loop) = main_loop_weak.upgrade() {
            main_loop.quit();
        }
    });
    let main_loop_weak = main_loop.downgrade();
    let _sig_term = main_loop.add_signal_local(Signal::SIGTERM, move || {
        if let Some(main_loop) = main_loop_weak.upgrade() {
            main_loop.quit();
        }
    });

    let context = pw::Context::new(&main_loop)?;
    // TODO: pass properties to connect
    let core = context.connect()?;

    let main_loop_weak = main_loop.downgrade();
    let _listener = core
        .add_listener_local()
        .info(|info| {
            dbg!(info);
        })
        .done(|_id, _seq| {
            // TODO
        })
        .error(move |id, seq, res, message| {
            eprintln!("error id:{} seq:{} res:{}: {}", id, seq, res, message);

            if id == 0 {
                if let Some(main_loop) = main_loop_weak.upgrade() {
                    main_loop.quit();
                }
            }
        })
        .register();

    let registry = Arc::new(core.get_registry());
    let registry_weak = Arc::downgrade(&registry);

    let _registry_listener = registry
        .add_listener_local()
        .global(move |obj| {
            if let Some(_registry) = registry_weak.upgrade() {
                match obj.type_ {
                    ObjectType::Node
                    | ObjectType::Port
                    | ObjectType::Module
                    | ObjectType::Device
                    | ObjectType::Factory
                    | ObjectType::Client
                    | ObjectType::Link => {
                        // TODO
                    }
                    _ => {
                        dbg!(obj);
                    }
                }
            }
        })
        .global_remove(|id| {
            println!("removed:");
            println!("\tid: {}", id);
        })
        .register();

    main_loop.run();

    Ok(())
}

fn main() -> Result<()> {
    pw::init();

    // TODO: add arguments

    monitor()?;

    unsafe {
        pw::deinit();
    }

    Ok(())
}
