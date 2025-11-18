# project_code

Small Rust example that calls a Hugging Face inference endpoint.

Usage

1. Create a `.env` file in the project root or set the `HF_TOKEN` environment variable.

Example `.env` (do not commit your real token):

```
HF_TOKEN=your_hf_token_here
```

2. Run the project:

```bash
cd project_code
export HF_TOKEN="your_hf_token_here" # or rely on .env
cargo run
```

What the program does

- Loads `.env` if present (via `dotenvy`).
- Builds a JSON request and sends it to the Hugging Face inference router with your token.
- Parses the response as JSON and pretty-prints it.

Security

- Do not commit your `.env` file. Add it to `.gitignore`.

Want changes?

I can update the program to save responses, retry on transient errors, or deserialize into a typed struct if you prefer.
