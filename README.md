# roles_for_reactions

![Rust CI](https://github.com/Celeo/roles_for_reactions/workflows/Rust%20CI/badge.svg?branch=master)

A [Discord](https://discord.com/) bot for allowing users to self-assign roles by adding reactions to a message.

## Installing

TBD

## Using

1. Create a [Discord app](https://discord.com/developers/applications)
1. Add a bot account to the app
1. Copy the bot's token, and put it into a file next to the binary called `.env` in the format `DISCORD_TOKEN=<your token here>`
1. Run the executable

## Developing

### Building

### Requirements

* Git
* A recent version of [Rust](https://www.rust-lang.org/tools/install)

### Steps

```sh
git clone https://github.com/Celeo/roles_for_reactions
cd roles_for_reactions
cargo build
```

If you have [just](https://github.com/casey/just) installed, just run `just` in the project root.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
* MIT license ([LICENSE-MIT](LICENSE-MIT))

## Contributing

Please feel free to contribute. Please open an issue first (or comment on an existing one) so that I know that you want to add/change something.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
