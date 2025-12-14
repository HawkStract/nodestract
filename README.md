# NodeStract (.hns)

> **Secure. Atomic. Abstract.**
> The proprietary programming language powering the HawkStract Ecosystem.

NodeStract (`.hns`) is a compiled, statically typed language designed for high-stakes environments where security, latency, and memory safety are non-negotiable. It powers the core infrastructure of HawkBank, HawkLock, and StractSound.

## 🚀 Key Features

### 1. Secure by Design (The Vault System)
Memory safety is not optional. NodeStract introduces `vault` variables, which are encrypted in RAM and decrypted only during CPU execution cycles.

```hns
// Password is obfuscated in memory dump
vault api_key = "sk_live_889900"

safe {
    // Only accessible within safe blocks
    Network.auth(api_key)
}
```

### 2. Capability-Based Security
A `.hns` file cannot access the network, file system, or system calls unless explicitly declared in the header. No more silent supply-chain attacks.

```hns
use capability Network { protocol: "https", domain: "*.hawkbank.com" }
// Any attempt to access "google.com" will fail at compile time.
```

### 3. Explicit Mutability
* `lock`: Immutable constants (Compile-time optimization).
* `stract`: Abstract mutable state.

## 🛠 Project Structure

* `/src` - Source code for the NodeStract Compiler (NSC).
* `/stdlib` - The Standard Library (HawkLock Crypto, HawkNet).
* `/examples` - Reference implementations.

## 📦 Getting Started

*Work in Progress. The compiler is currently in pre-alpha stage.*

To run the syntax parser (soon):
```bash
./bin/nsc build main.hns
```

## 🤝 Contributing

We are building the future of secure computing.
1.  Fork the project.
2.  Create your feature branch (`git checkout -b feature/amazing-feature`).
3.  Commit your changes.
4.  Open a Pull Request.

## 📄 License

Distributed under the MIT License. See `LICENSE` for more information.

---
**HawkStract Ecosystem**