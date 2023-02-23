This is a basic latex bot for matrix written in rust. It allows rendering latex code as images on matrix.

# Setup
For the initial setup one has to simply run the bot with `./matrix_latex_bot` and follow the bot's instructions in the command line. 
At first it asks for a password to encrypt the encryption store containing the encryption keys and other session relevant data with.
This prompt doesn't appear if the `MATRIX_BOT_CRYPTO_PW` environment variable is set to the desired password value already.
After this the bot will prompt for authentication with the bot's home server, username and password.

On later startups only entering the encryption password or setting `MATRIX_BOT_CRYPTO_PW` is needed, due to the bot saving the required login token and 
other data to bot_credential.yml in its working directory.

# Commands

| Command | Parameters| Function
|---|---|---
| ping | None | Replies with with a pong message.
| halt | None | Shuts the bot down.
| tex  | latex code | Replies with with an image containing the typeset latex code or an error message.
| math | latex math code | Replies with with an image containing the typeset latex math code or an error message.

# License
The bot is licensed under AGPL-3.0, which can be found in the [license file](./LICENSE)
