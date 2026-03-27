<div align="center">

# Folio

### Wealth Portfolio Management

**Track investments with automatic Australian bank statement import**

[![GitHub](https://img.shields.io/badge/GitHub-181717?style=for-the-badge&logo=github&logoColor=white)](https://github.com/Sensible-Analytics/folio)

</div>

---

## 🛡️ Security First

> ⚠️ **CRITICAL SECURITY WARNING**
>
> This repository uses **automated secret scanning**. NEVER commit:
> - API keys (OpenAI, Anthropic, database credentials)
> - AI agent tokens
> - Database connection strings
> - Private keys
>
> **Before committing:** Review our [Security Policy](SECURITY.md) and [AI Agent Keys Policy](AI_AGENT_KEYS_POLICY.md)

---

## 🎯 What is Folio?

Folio is a **personal wealth management application** designed for Australian investors. It combines portfolio tracking with automatic bank statement import, making it easy to monitor your investments and spending in one place.

### Why Folio?

- 🇦🇺 **Built for Australia** — Native support for Australian banks
- 📊 **Unified View** — Investments and bank accounts in one dashboard
- 🏦 **Auto-Import** — Connect your bank statements automatically
- 📈 **Performance Tracking** — Real-time portfolio analytics
- 💰 **Net Worth Calculator** — Track your complete financial picture

---

## ✨ Features

### Portfolio Management

- **Multi-Asset Support** — Shares, ETFs, managed funds, property
- **Performance Analytics** — Track returns, dividends, and distributions
- **Tax Reporting** — Capital gains and income summaries
- **Benchmark Comparison** — Compare against ASX indices
- **Dividend Tracking** — Automatic dividend reinvestment calculations

### Bank Integration

- **Australian Banks** — Support for major Australian banks
- **Statement Import** — Automatic CSV/OFX import
- **Transaction Categorization** — AI-powered spending categorization
- **Cash Flow Analysis** — Income vs expenses tracking
- **Reconciliation** — Match transactions across accounts

### Reporting

- **Portfolio Reports** — Detailed performance summaries
- **Tax Statements** — Annual tax reporting ready
- **Cash Flow Reports** — Track where your money goes
- **Custom Date Ranges** — Analyze any time period
- **Export Options** — PDF, CSV, and Excel exports

---

## 🚀 Quick Start

### Prerequisites

- Node.js 18+
- A modern web browser

### Installation

```bash
# Clone the repository
git clone https://github.com/Sensible-Analytics/folio.git
cd folio

# Install dependencies
npm install

# Set up environment variables
cp .env.example .env
# Edit .env with your configuration

# Run development server
npm run dev
```

Visit `http://localhost:3000` to access the application.

---

## 🛠️ Tech Stack

| Component | Technology |
|-----------|------------|
| **Frontend** | TypeScript, React |
| **Styling** | Tailwind CSS |
| **State Management** | Zustand |
| **Charts** | Recharts |
| **Build Tool** | Vite |

---

## 📊 Supported Banks

Folio supports automatic import from major Australian banks:

- Commonwealth Bank
- Westpac
- ANZ
- NAB
- ING
- Macquarie
- And more...

---

## 🔒 Privacy & Security

- **Local-First** — Your data stays on your device
- **No Cloud Storage** — We don't store your financial data
- **Open Source** — Audit the code yourself
- **No Tracking** — No analytics or tracking

---

## 🔐 Development Security

### 🚨 Security Requirements

This repository includes **automated secret scanning**. NEVER commit:
- API keys or tokens
- Database credentials
- Private keys

**Before contributing:**

1. **Install pre-commit hooks:**
   ```bash
   pip install pre-commit
   pre-commit install
   ```

2. **Use environment variables:**
   ```bash
   cp .env.example .env
   # Edit .env (NEVER commit!)
   ```

3. **If you expose a secret:**
   - Revoke immediately
   - Contact: security@sensibleanalytics.co

See [Security Policy](SECURITY.md) and [AI Agent Keys Policy](AI_AGENT_KEYS_POLICY.md) for details.

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md).

---

## ⚠️ Disclaimer

Folio is a personal finance tool and is **not financial advice**. Always consult with a qualified financial advisor before making investment decisions.

---

## 📄 License

MIT License — see [LICENSE](LICENSE)

---

<div align="center">

**Built by [Sensible Analytics](https://www.sensibleanalytics.co)**  
*AI architecture for regulated industries*

</div>
