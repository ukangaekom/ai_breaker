# ai_breaker


## AI Breaker: Embedded LLM Security Evaluation Framework

A Rust-based, embedded LLM security evaluation framework that interacts with multiple AI models, allows configurable system prompts, performs prompt-injection/jailbreak experiments, records complete conversations and metadata (ULID, timestamps, latency, tokens, model version, etc.) in redb, and is intended to produce reproducible evidence for AI security research.

### Why it’s powerful

- **CLI installable**: use it like any other terminal tool, with clean commands and easy local setup.
- **Unified pipeline**: one interface to orchestrate prompts, responses, and multi-agent flows across providers.
- **Multi-agent testing**: built to run many model agents together, compare output, and validate behavior from a single binary.
- **Real-time terminal interaction**: live input, colored output, and prompt-driven workflows for rapid experimentation.

### Supported backends

AI Breaker supports a huge ecosystem of providers, including:

`openai`, `openai_resp`, `anthropic`, `gemini`, `omlx`, `ollama`, `ollama_cloud`, `vertex`, `bedrock_api`, `bedrock_sigv4`, `github_copilot`, `opencode_go`, `groq`, `together`, `fireworks`, `cohere`, `nebius`, `mimo`, `deepseek`, `minimax`, `zai`, `zai_coding`, `bigmodel`, `aliyun`, `baidu`, `moonshot`, `kimi`, `aihubmix`, `open_router`, `atlascloud`, `xai`

### What makes it excellent

- **Single CLI entrypoint** for model orchestration
- **Consistent command syntax** across providers
- **Fast local install** and execution
- **Perfect for benchmarking, prompt engineering, and debugging**
- **Easy to extend** with new model adapters

> AI Breaker is the CLI tool you want when you need a unified, installable engine for testing a diverse set of LLM providers from the terminal.