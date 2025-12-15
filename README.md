# Node Stract (.ns)

> **Secure. Atomic. Abstract.**
> The proprietary programming language powering the HawkStract Ecosystem.

**Node Stract** (.ns) is a compiled, statically typed language designed for high-stakes environments where security, latency, and memory safety are non-negotiable. It powers the core infrastructure of HawkBank, HawkLock, and StractSound.

## Key Features

### 1. Secure by Design (The Vault System)
Memory safety is not optional. Node Stract introduces vault variables, which are encrypted in RAM (AES-256-GCM) and decrypted only during CPU execution cycles. Even a memory dump won't reveal your secrets.

Esempio:
vault api_key = "sk_live_889900";
stract counter = 0;

### 2. Capability-Based Security
A .ns file cannot access system IO, Filesystem, or Network unless explicitly declared in the header. No more silent supply-chain attacks.

Esempio:
- use IO {}
- use Network {}

### 3. Explicit State
- lock: Immutable constants.
- stract: Abstract mutable state.

## Installation & Setup

### Prerequisites
- Rust Toolchain (cargo)

### 1. Clone the Repository
git clone [https://github.com/HawkStract/nodestract.git](https://github.com/HawkStract/nodestract.git)
cd nodestract

### 2. Build the Compiler (NSC)
cargo build --release

L'eseguibile del compilatore si troverà in ./target/release/nsc.exe (Windows) o ./target/release/nsc (Linux/Mac).

### 3. Add to Path (Optional)
Aggiungi la cartella target/release al PATH di sistema per eseguire nsc da qualsiasi terminale.

## Usage

### Create a Hello World
Crea un file chiamato hello.ns:

module Hello;
use IO {}

func main() {
    lock message = "Hello, Node Stract!";
    IO.print(message);
}

### Compile and Run
Esegui il compilatore contro il tuo file:

nsc build hello.ns

## Examples

Controlla la cartella /examples per utilizzi avanzati:
- math_test.ns: Operazioni matematiche e promozione dei tipi.
- security_test.ns: Dimostrazione della cifratura Vault e immutabilità Lock.
- data_test.ns: Utilizzo di Array e Mappe (Dizionari).

## Contributing

Stiamo costruendo il futuro del calcolo sicuro.
1. Fai il Fork del progetto.
2. Crea il tuo feature branch.
3. Committa i tuoi cambiamenti.
4. Apri una Pull Request.

### Visual Identity (Windows)
Per visualizzare correttamente l'icona di NodeStract sui file `.ns`, esegui lo script di configurazione:
1. Apri PowerShell nella cartella del progetto.
2. Esegui: `./setup_env.ps1`

## License

Distribuito sotto licenza MIT. Vedi il file LICENSE per maggiori informazioni.

---
**HawkStract Ecosystem**