

```bash
# Create sqlite database
sqlx database create

# Build up tables
sqlx migrate run --source ./senra_server/migrations
```