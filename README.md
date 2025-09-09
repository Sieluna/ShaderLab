# ShaderLab

## Development Setup

### Database Setup

```bash
# Create sqlite database
sqlx database create

# Build up tables
sqlx migrate run --source ./senra_server/migrations
```

## Production Deployment

The `deploy.sh` script provides automated deployment and management the server

### Quick Deployment

```bash
curl -sSL https://raw.githubusercontent.com/Sieluna/ShaderLab/refs/heads/develop/deploy.sh | sudo bash -s install
```
