# Event Bus

An event system written in Rust written with thread-safety in mind.

* Dispatch custom events to buses.
* Subscribe to receive events from a bus.
* Can use priority to order event subscribers.
* Events can be cancelled or modified.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/event-bus.svg
[crates-url]: https://crates.io/crates/event-bus
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE