# 🪙 SPLTokenMint

A Solana Anchor-based program to mint custom tokens using the Token-2022 program with optional extensions like:

- Transfer fees
- Memo transfer enforcement
- Default frozen accounts
- Immutable ownership
- Initial supply + auto ATA creation

---

## 🚀 Features

✅ Create a custom token with dynamic settings  
✅ Add optional extensions conditionally  
✅ Supports minting initial supply to user's ATA  
✅ Creates ATA automatically if not exists  
✅ Clean, modular Anchor layout  

---

## 🛠️ Program Structure

- `programs/custom_token/`: Anchor smart contract
- `tests/`: JavaScript test suite (optional)
- `CreateTokenArgs`: Struct with parameters like decimals, fees, and flags
- `CreateTokenWithExtensions`: Instruction to create and initialize the mint

---

## 🧪 Usage Example (Client-side)

_Coming soon..._

This will include:
- How to call the program from a frontend
- How to send `CreateTokenArgs`
- How to fetch the resulting mint address

---

## 📚 Token Extensions Used

| Extension              | Purpose                               |
|------------------------|----------------------------------------|
| `TransferFeeConfig`    | Configures fee parameters              |
| `TransferFeeAmount`    | Tracks fee amounts                     |
| `MemoTransfer`         | Requires memo in transfers             |
| `DefaultAccountState`  | Sets default state (e.g., frozen)      |
| `ImmutableOwner`       | Makes mint owner unchangeable          |

---

## 📦 Requirements

- Anchor
- Solana CLI
- SPL Token 2022
- Node.js (if using frontend or test scripts)

---

## ✍️ License

MIT — feel free to fork, build, and contribute!

---

## 🤝 Contributing

PRs welcome. Open an issue if you find a bug or want to suggest a feature.
