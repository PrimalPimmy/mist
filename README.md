# mist

Mist is a Discord bot written in Rust using the Serenity library. It provides some utility commands. I plan to make it a more arcade type bot, hopefully. It's part of my process of learning bot code design

## Features

- **Ping**: Responds with "Pong!" to the `!ping` command.
- **Snipe**: The `msnipe` command displays the author, content, and timestamp of the most recently deleted message in the channel.
- **Caching**: Implements a local message cache to track recent messages for reliable sniping.

## Setup

### Prerequisites

- Rust and Cargo
- A Discord Bot Application with a valid token

### Configuration

The bot requires the `DISCORD` environment variable to be set with your bot token.

### Running the Bot

1. Clone the repository.
2. Set the environment variable:
   ```sh
   export DISCORD=your_bot_token_here
   ```
3. Run the project using Cargo:
   ```sh
   cargo run
   ```
