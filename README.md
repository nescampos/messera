# Messera

Messera is a direct messaging system between chains (users) to be able to interact through text, replicating the WhatsApp or Telegram format, but on the Web3, maintaining privacy, confidentiality, and most importantly, the ownership of messages, using [**Linera Protocol**](https://linera.dev/).

**Messera** use [Linera microchains](https://linera.io/whitepaper) (via Rust)

## How works

Demo: https://youtu.be/sADDg2Ar5kA

1. Clone the project:
```sh
git clone https://github.com/nescampos/messera.git
```

2. Install and run Linera: https://linera.dev/getting_started/installation.html
3. Run Messera: 

```sh
linera --storage $LINERA_STORAGE --wallet $LINERA_WALLET publish-and-create \
  <path to messera folder>/target/wasm32-unknown-unknown/release/messera_{contract,service}.wasm 
```

