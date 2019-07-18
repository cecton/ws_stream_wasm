//! # ws_stream_wasm
//!
//! [![standard-readme compliant](https://img.shields.io/badge/readme%20style-standard-brightgreen.svg?style=flat-square)](https://github.com/RichardLitt/standard-readme)
//! [![Build Status](https://api.travis-ci.org/najamelan/ws_stream_wasm.svg?branch=master)](https://travis-ci.org/najamelan/ws_stream_wasm)
//! [![Docs](https://docs.rs/ws_stream_wasm/badge.svg)](https://docs.rs/ws_stream_wasm)
//! [![crates.io](https://img.shields.io/crates/v/ws_stream_wasm.svg)](https://crates.io/crates/ws_stream_wasm)
//!
//!
//! > A convenience library for using websockets in WASM
//!
//! **features:**
//! - `WsStream`  : A wrapper around [`web_sys::WebSocket`](https://docs.rs/web-sys/0.3.25/web_sys/struct.WebSocket.html).
//! - `WsMessage` : A simple rusty representation of a WebSocket message.
//! - `WsIo`      : A futures Sink/Stream of WsMessage. (can use the futures compat layer to get futures 01 versions).
//!                 It also implements AsyncRead/AsyncWrite from futures 0.3. With the compat layer you can obtain futures
//!                 01 versions for use with tokio codec.
//!
//! **NOTE:** this crate only works on WASM. If you want a server side equivalent that implements AsyncRead/AsyncWrite over
//! WebSockets, check out [ws_stream](https://crates.io/crates/ws_stream).
//!
//! **missing features:**
//! - no automatic reconnect
//! - no events (probably I'll make it Observable with [pharos](https://crates.io/crates/pharos) one day)
//! - not all features are thoroughly tested. Notably, I have little use for extensions and subprotocols. Tungstenite,
//!   which I use for the server end (and for automated testing) doesn't support these, making it hard to write unit tests.
//!
//! ## Table of Contents
//!
//! - [Install](#install)
//!   - [Dependencies](#dependencies)
//! - [Usage](#usage)
//! - [API](#api)
//! - [Contributing](#contributing)
//!   - [Code of Conduct](#code-of-conduct)
//! - [License](#license)
//!
//!
//! ## Install
//! With [cargo add](https://github.com/killercup/cargo-edit):
//! `cargo add ws_stream_wasm`
//!
//! With [cargo yaml](https://gitlab.com/storedbox/cargo-yaml):
//! ```yaml
//! dependencies:
//!
//!   ws_stream_wasm: ^0.1
//! ```
//!
//! With raw Cargo.toml
//! ```toml
//! [dependencies]
//!
//!    ws_stream_wasm = "^0.1"
//! ```
//!
//! ### Dependencies
//!
//! This crate has few dependiencies. Cargo will automatically handle it's dependencies for you.
//!
//! There are no optional features.
//!
//! ## Usage
//!
//! Please have a look in the [examples directory of the repository](https://github.com/najamelan/ws_stream_wasm/tree/master/examples).
//!
//! The [integration tests](https://github.com/najamelan/ws_stream_wasm/tree/master/tests) are also useful.
//!
//! ## API
//!
//! Api documentation can be found on [docs.rs](https://docs.rs/ws_stream_wasm).
//!
//!
//! ## Contributing
//!
//! This repository accepts contributions. Ideas, questions, feature requests and bug reports can be filed through github issues.
//!
//! Pull Requests are welcome on github. By commiting pull requests, you accept that your code might be modified and
//! reformatted to fit the project coding style or to improve the implementation. Please discuss what you want to
//! see modified before filing a pull request if you don't want to be doing work that might be rejected.
//!
//! ### Testing
//!
//! For testing we need backend servers to echo data back to the tests. These are in the `ws_stream` crate.
//! ```shell
//! git clone https://github.com/najamelan/ws_stream
//! cd ws_stream
//! cargo run --expample echo --release
//!
//! # in a different terminal:
//! cargo run --example echo_tt --release -- "127.0.0.1:3312"
//!
//! # the second server is pure tokio-tungstenite without ws_stream wrapping it in AsyncRead/Write. This
//! # is needed for testing a WsMessage::Text because ws_stream only does binary.
//!
//! # in a third terminal, in ws_stream_wasm you have different options:
//! wasm-pack test --firefox [--headless] [--release]
//! wasm-pack test --chrome  [--headless] [--release]
//! ```
//!
//! In general chrome is well faster. When running it in the browser (without `--headless`) you get trace logging
//! in the console, which helps debugging. In chrome you need to enable verbose output in the console,
//! otherwise only info and up level are reported.
//!
//! ### Code of conduct
//!
//! Any of the behaviors described in [point 4 "Unacceptable Behavior" of the Citizens Code of Conduct](http://citizencodeofconduct.org/#unacceptable-behavior) are not welcome here and might get you banned.
//! If anyone including maintainers and moderators of the project fail to respect these/your limits,
//! you are entitled to call them out.
//!
//! ## License
//!
//! [Unlicence](https://unlicense.org/)
//
#![ doc    ( html_root_url = "https://docs.rs/ws_stream_wasm" ) ]
#![ feature( async_await                                      ) ]
#![ deny   ( missing_docs                                     ) ]
#![ forbid ( unsafe_code                                      ) ]
#![ allow  ( clippy::suspicious_else_formatting               ) ]

mod error           ;
mod ws_message      ;
mod ws_io           ;
mod ws_state        ;
mod ws_stream       ;
mod callback_future ;

pub use
{
	ws_state          :: { WsState                } ,
	callback_future   :: { future_event           } ,
	error             :: { WsErr      , WsErrKind } ,
	ws_message        :: { WsMessage              } ,
	ws_io             :: { WsIo                   } ,
	ws_stream         :: { WsStream               } ,
};



mod import
{
	pub(crate) use
	{
		async_runtime :: { rt                                                                      } ,
		failure       :: { Backtrace, Fail, Context as FailContext                                 } ,
		futures       :: { channel::mpsc::unbounded, Poll                                          } ,
		futures       :: { prelude::{ Stream, Sink, AsyncWrite, AsyncRead }, stream::{ StreamExt } } ,
		futures       :: { ready                                                                   } ,
		std           :: { io, cmp, collections::VecDeque, fmt, task::{ Context, Waker }           } ,
		std           :: { rc::Rc, cell::{ RefCell }, pin::Pin, convert::{ TryFrom, TryInto }      } ,
		log           :: { *                                                                       } ,
		js_sys        :: { ArrayBuffer, Uint8Array                                                 } ,
		wasm_bindgen  :: { closure::Closure, JsCast, JsValue, UnwrapThrowExt                       } ,
		web_sys       :: { *, BinaryType, Blob, WebSocket                                          } ,
		js_sys        :: { Array                                                                   } ,
	};
}
