# Quick Reference Commands

## Starting the Database (After Shutdown/Reboot)

Navigate to project directory and start:
```bash
cd /home/pradeep/citytech_rust/my-fintech-app
docker compose up -d
```

Or use the quick start script:
```bash
cd /home/pradeep/citytech_rust/my-fintech-app
./start-db.sh
```

## Check Database Status

Check if container is running:
```bash
docker compose ps
```

## View Tables

List all tables:
```bash
docker exec fintech_db psql -U fintech_user -d fintech_db -c "\dt"
```

View specific table structure:
```bash
docker exec fintech_db psql -U fintech_user -d fintech_db -c "\d users"
docker exec fintech_db psql -U fintech_user -d fintech_db -c "\d wallets"
docker exec fintech_db psql -U fintech_user -d fintech_db -c "\d transactions"
```

## Interactive Database Access

Connect to database shell:
```bash
docker exec -it fintech_db psql -U fintech_user -d fintech_db
```

Once inside, you can run SQL commands:
```sql
\dt                    -- List tables
\d users              -- Describe users table
SELECT * FROM users;  -- Query users
\q                    -- Quit
```

## Stop Database

Stop but keep data:
```bash
docker compose down
```

Stop and remove all data (⚠️ destructive):
```bash
docker compose down -v
```

## View Logs

```bash
docker compose logs postgres
docker compose logs -f postgres  # Follow logs in real-time
```

## Database Connection Info

- **Host**: localhost
- **Port**: 5433
- **Database**: fintech_db
- **User**: fintech_user
- **Password**: fintech_password
- **Connection String**: `postgresql://fintech_user:fintech_password@localhost:5433/fintech_db`
