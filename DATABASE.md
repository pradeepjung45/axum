# Database Setup Guide

## Quick Start

Start the PostgreSQL database:
```bash
docker compose up -d
```

Stop the database:
```bash
docker compose down
```

Stop and remove all data:
```bash
docker compose down -v
```

## Database Details

- **Host**: localhost
- **Port**: 5433 (mapped to avoid conflicts)
- **Database**: fintech_db
- **User**: fintech_user
- **Password**: fintech_password

## Schema

The database includes three main tables:

### Users
- `id` (UUID, Primary Key)
- `email` (VARCHAR, Unique)
- `password_hash` (VARCHAR)
- `full_name` (VARCHAR)
- `created_at`, `updated_at` (Timestamps)

### Wallets
- `id` (UUID, Primary Key)
- `user_id` (UUID, Foreign Key → users)
- `balance` (DECIMAL, must be >= 0)
- `currency` (VARCHAR, default 'USD')
- `created_at`, `updated_at` (Timestamps)

### Transactions
- `id` (UUID, Primary Key)
- `wallet_id` (UUID, Foreign Key → wallets)
- `transaction_type` (ENUM: DEPOSIT, WITHDRAWAL, TRANSFER)
- `amount` (DECIMAL, must be > 0)
- `description` (TEXT)
- `status` (ENUM: PENDING, COMPLETED, FAILED)
- `created_at` (Timestamp)

## Useful Commands

Connect to the database:
```bash
docker exec -it fintech_db psql -U fintech_user -d fintech_db
```

List all tables:
```sql
\dt
```

View table structure:
```sql
\d users
\d wallets
\d transactions
```

Check database logs:
```bash
docker compose logs postgres
```
