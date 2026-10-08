# Security Policy

## Scope & Deployment Status

* **Status**: Experimental MVP (`v0.1.0`).
* **Environment**: Stellar Testnet only.
* **Audit Status**: Unaudited. No third-party professional security audit has been conducted.
* **Mainnet Notice**: This contract is NOT intended for production mainnet deployment with real financial assets.

## Custody & Signing Boundaries

* SubPath is strictly **non-custodial**. The contract does not hold user balances or maintain custody of private keys.
* Token transfers use standard SEP-41 token allowances.
* All state-changing methods except `execute_billing` require cryptographic authentication from the respective account.

## Reporting a Vulnerability

If you discover a security vulnerability or potential exploit, please report it responsibly:

* **Email**: `security@subpath-protocol.com`
* **Do Not File Public Issues**: Do not disclose vulnerabilities in public GitHub issues or discussions.
* **Response Commitment**: We acknowledge reports within 48 hours and coordinate fixes prior to public disclosure.
